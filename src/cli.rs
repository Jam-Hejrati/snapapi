//! Synchronous command-line boundary: arguments, configuration, I/O, and exit status.

use crate::exit::Exit;
use crate::policy::FailOn;
use crate::{compare, input, model, report};
use clap::{Arg, ArgAction, Command, value_parser};
use serde::Deserialize;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Config {
    version: Option<u8>,
    policy: PolicyConfig,
    output: OutputConfig,
    comparison: ComparisonConfig,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct PolicyConfig {
    fail_on: Option<String>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct OutputConfig {
    format: Option<String>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct ComparisonConfig {
    include_informational: bool,
    max_bytes: Option<u64>,
    unresolved_references: Option<String>,
}

struct CliError {
    exit: Exit,
    message: String,
}
impl CliError {
    fn usage(message: impl ToString) -> Self {
        Self {
            exit: Exit::Usage,
            message: message.to_string(),
        }
    }
    fn input(message: impl ToString) -> Self {
        Self {
            exit: Exit::Input,
            message: message.to_string(),
        }
    }
    fn internal(message: impl ToString) -> Self {
        Self {
            exit: Exit::Internal,
            message: message.to_string(),
        }
    }
}

fn command() -> Command {
    Command::new("snapapi")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Detect potentially breaking OpenAPI contract changes")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(Command::new("version").about("Print the version"))
        .subcommand(
            Command::new("compare")
                .about("Compare two local OpenAPI documents")
                .arg(
                    Arg::new("baseline")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("candidate")
                        .required(true)
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("format")
                        .long("format")
                        .value_parser(["text", "json", "markdown"]),
                )
                .arg(Arg::new("fail-on").long("fail-on").value_parser([
                    "none",
                    "breaking",
                    "review",
                    "any-change",
                ]))
                .arg(
                    Arg::new("config")
                        .long("config")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("output")
                        .long("output")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("max-bytes")
                        .long("max-bytes")
                        .value_parser(value_parser!(u64)),
                )
                .arg(Arg::new("quiet").long("quiet").action(ArgAction::SetTrue))
                .arg(
                    Arg::new("verbose")
                        .long("verbose")
                        .action(ArgAction::SetTrue),
                ),
        )
}

pub fn run<I, T>(args: I) -> Exit
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let matches = match command().try_get_matches_from(args) {
        Ok(matches) => matches,
        Err(error) => {
            let exit = if error.use_stderr() {
                Exit::Usage
            } else {
                Exit::Ok
            };
            let _ = error.print();
            return exit;
        }
    };
    match execute(&matches) {
        Ok(exit) => exit,
        Err(error) => {
            eprintln!("snapapi: {}", error.message);
            error.exit
        }
    }
}

fn execute(matches: &clap::ArgMatches) -> Result<Exit, CliError> {
    if matches.subcommand_name() == Some("version") {
        write_stdout(&format!("snapapi {}\n", env!("CARGO_PKG_VERSION")))?;
        return Ok(Exit::Ok);
    }
    let args = matches
        .subcommand_matches("compare")
        .ok_or_else(|| CliError::usage("missing compare command"))?;
    let config = if let Some(path) = args.get_one::<PathBuf>("config") {
        let bytes = input::read_limited(path, 1024 * 1024).map_err(CliError::usage)?;
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| CliError::usage("configuration is not valid UTF-8"))?;
        let raw = if path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            serde_json::from_str::<crate::json::UniqueValue>(text)
                .map(|v| v.0)
                .map_err(|_| CliError::usage("invalid JSON configuration"))?
        } else {
            serde_saphyr::from_str::<serde_json::Value>(text)
                .map_err(|_| CliError::usage("invalid YAML configuration"))?
        };
        let config: Config = serde_json::from_value(raw)
            .map_err(|_| CliError::usage("invalid configuration field or unknown key"))?;
        if config.version != Some(1) {
            return Err(CliError::usage("configuration version must be 1"));
        }
        config
    } else {
        Config::default()
    };
    // Validate configuration even when a CLI override would mask a bad value.
    if let Some(value) = &config.policy.fail_on {
        parse_policy(value)?;
    }
    if config
        .output
        .format
        .as_deref()
        .is_some_and(|v| !matches!(v, "text" | "json" | "markdown"))
    {
        return Err(CliError::usage("invalid output format in configuration"));
    }
    if config
        .comparison
        .unresolved_references
        .as_deref()
        .is_some_and(|v| v != "error")
    {
        return Err(CliError::usage("unresolved_references supports only error"));
    }
    let format = args
        .get_one::<String>("format")
        .map(String::as_str)
        .or(config.output.format.as_deref())
        .unwrap_or("text");
    let policy = parse_policy(
        args.get_one::<String>("fail-on")
            .map(String::as_str)
            .or(config.policy.fail_on.as_deref())
            .unwrap_or("breaking"),
    )?;
    let max_bytes = args
        .get_one::<u64>("max-bytes")
        .copied()
        .or(config.comparison.max_bytes)
        .unwrap_or(input::DEFAULT_MAX_BYTES);
    let options = input::LoadOptions { max_bytes };
    let baseline_path = args
        .get_one::<PathBuf>("baseline")
        .ok_or_else(|| CliError::usage("missing baseline"))?;
    let candidate_path = args
        .get_one::<PathBuf>("candidate")
        .ok_or_else(|| CliError::usage("missing candidate"))?;
    let baseline =
        model::normalize(&input::load(baseline_path, &options).map_err(CliError::input)?)
            .map_err(CliError::input)?;
    let candidate =
        model::normalize(&input::load(candidate_path, &options).map_err(CliError::input)?)
            .map_err(CliError::input)?;
    let format = match format {
        "json" => report::Format::Json,
        "markdown" => report::Format::Markdown,
        _ => report::Format::Text,
    };
    let report = report::Report::new(
        &baseline,
        &candidate,
        compare::compare(&baseline, &candidate),
        policy,
    );
    let rendered = report
        .render(
            format,
            args.get_flag("verbose") || config.comparison.include_informational,
        )
        .map_err(CliError::internal)?;
    if let Some(path) = args.get_one::<PathBuf>("output") {
        if same_file(path, baseline_path)
            || same_file(path, candidate_path)
            || args
                .get_one::<PathBuf>("config")
                .is_some_and(|config| same_file(path, config))
        {
            return Err(CliError::usage(
                "output path must not overwrite an input or configuration file",
            ));
        }
        std::fs::write(path, rendered).map_err(CliError::internal)?;
    } else if !args.get_flag("quiet") {
        write_stdout(&rendered)?;
    } else {
        for diagnostic in &report.diagnostics {
            eprintln!(
                "Unsupported {} at {}:{}",
                diagnostic.feature,
                diagnostic.source.display(),
                diagnostic.location
            );
        }
    }
    Ok(if report.policy_violated {
        Exit::PolicyViolated
    } else {
        Exit::Ok
    })
}

fn same_file(a: &std::path::Path, b: &std::path::Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(a) = std::fs::metadata(a)
            && let Ok(b) = std::fs::metadata(b)
            && a.dev() == b.dev()
            && a.ino() == b.ino()
        {
            return true;
        }
    }
    a == b
        || a.canonicalize()
            .ok()
            .zip(b.canonicalize().ok())
            .is_some_and(|(a, b)| a == b)
}

fn parse_policy(value: &str) -> Result<FailOn, CliError> {
    match value {
        "none" => Ok(FailOn::None),
        "breaking" => Ok(FailOn::Breaking),
        "review" => Ok(FailOn::Review),
        "any-change" => Ok(FailOn::AnyChange),
        _ => Err(CliError::usage("invalid fail_on policy")),
    }
}

fn write_stdout(text: &str) -> Result<(), CliError> {
    std::io::stdout()
        .lock()
        .write_all(text.as_bytes())
        .map_err(CliError::internal)
}
