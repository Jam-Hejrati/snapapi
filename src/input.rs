//! Loading OpenAPI documents from local files.
//!
//! This stage reads a file with a size limit, works out whether it is JSON or
//! YAML, parses it into a [`serde_json::Value`], and checks that it declares a
//! supported OpenAPI version. Both formats end up as the same `Value`, so the
//! rest of the pipeline never needs to know which one it came from.

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde_json::Value;

/// Default maximum input size: 32 MiB.
pub const DEFAULT_MAX_BYTES: u64 = 32 * 1024 * 1024;

/// Serialization format of an input document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Yaml,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Format::Json => "JSON",
            Format::Yaml => "YAML",
        })
    }
}

/// OpenAPI major/minor versions SnapAPI understands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenApiVersion {
    V3_0,
    V3_1,
}

/// A parsed document that has passed the version check.
#[derive(Debug, Clone)]
pub struct Document {
    /// Where the document came from, used in diagnostics and reports.
    pub source: PathBuf,
    pub format: Format,
    pub version: OpenApiVersion,
    /// The `openapi` field exactly as written, e.g. `"3.0.3"`.
    pub version_string: String,
    pub root: Value,
}

/// Options for [`load`].
#[derive(Debug, Clone)]
pub struct LoadOptions {
    pub max_bytes: u64,
}

impl Default for LoadOptions {
    fn default() -> Self {
        LoadOptions {
            max_bytes: DEFAULT_MAX_BYTES,
        }
    }
}

/// Why an input could not be loaded. Every variant maps to exit code 3.
#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("{}: cannot read file: {source}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("{}: file is larger than the {limit}-byte limit", path.display())]
    TooLarge { path: PathBuf, limit: u64 },

    #[error("{}: file is not valid UTF-8", path.display())]
    NotUtf8 { path: PathBuf },

    #[error("{}{}: invalid {format}: {message}", path.display(), fmt_location(*line, *column))]
    Parse {
        path: PathBuf,
        format: Format,
        message: String,
        line: Option<u64>,
        column: Option<u64>,
    },

    #[error("{}: expected the document root to be an object", path.display())]
    NotAnObject { path: PathBuf },

    #[error("{}: Swagger 2.0 documents are not supported; convert to OpenAPI 3.0 or 3.1", path.display())]
    Swagger2 { path: PathBuf },

    #[error("{}: missing the `openapi` version field", path.display())]
    MissingVersion { path: PathBuf },

    #[error(
        "{}: the `openapi` field must be a string such as \"3.0.3\" (in YAML, quote it so it is not read as a number)",
        path.display()
    )]
    VersionNotString { path: PathBuf },

    #[error("{}: unsupported OpenAPI version \"{found}\"; SnapAPI supports 3.0.x and 3.1.x", path.display())]
    UnsupportedVersion { path: PathBuf, found: String },
}

fn fmt_location(line: Option<u64>, column: Option<u64>) -> String {
    match (line, column) {
        (Some(l), Some(c)) => format!(":{l}:{c}"),
        (Some(l), None) => format!(":{l}"),
        _ => String::new(),
    }
}

/// Reads, parses and version-checks the document at `path`.
pub fn load(path: &Path, options: &LoadOptions) -> Result<Document, InputError> {
    let bytes = read_limited(path, options.max_bytes)?;
    let text = String::from_utf8(bytes).map_err(|_| InputError::NotUtf8 {
        path: path.to_owned(),
    })?;
    parse(path, &text)
}

/// Parses already-loaded text. `source` is used for format detection by
/// extension and for diagnostics.
pub fn parse(source: &Path, text: &str) -> Result<Document, InputError> {
    // Ignore a UTF-8 byte-order mark; editors on Windows often add one.
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let format = detect_format(source, text);
    let root = match format {
        Format::Json => parse_json(source, text)?,
        Format::Yaml => parse_yaml(source, text)?,
    };
    let (version, version_string) = check_version(source, &root)?;
    Ok(Document {
        source: source.to_owned(),
        format,
        version,
        version_string,
        root,
    })
}

fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>, InputError> {
    let read_err = |source| InputError::Read {
        path: path.to_owned(),
        source,
    };
    let file = File::open(path).map_err(read_err)?;
    let mut bytes = Vec::new();
    // Read one byte past the limit, so an oversized file is detected even if
    // its metadata is wrong (pipes, special files, files still being written).
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(read_err)?;
    if bytes.len() as u64 > limit {
        return Err(InputError::TooLarge {
            path: path.to_owned(),
            limit,
        });
    }
    Ok(bytes)
}

/// Picks the format from the file extension first, then from the content.
fn detect_format(source: &Path, text: &str) -> Format {
    let ext = source
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("json") => Format::Json,
        Some("yaml" | "yml") => Format::Yaml,
        // A JSON document must start with `{` once whitespace is skipped; a
        // YAML OpenAPI document almost never does.
        _ if text.trim_start().starts_with('{') => Format::Json,
        _ => Format::Yaml,
    }
}

fn parse_json(source: &Path, text: &str) -> Result<Value, InputError> {
    serde_json::from_str(text).map_err(|e| InputError::Parse {
        path: source.to_owned(),
        format: Format::Json,
        message: strip_json_location(&e.to_string()),
        line: Some(e.line() as u64),
        column: Some(e.column() as u64),
    })
}

/// `serde_json` appends " at line X column Y" to its messages; we report the
/// location ourselves, in `path:line:col` form.
fn strip_json_location(message: &str) -> String {
    match message.rfind(" at line ") {
        Some(i) => message[..i].to_owned(),
        None => message.to_owned(),
    }
}

fn parse_yaml(source: &Path, text: &str) -> Result<Value, InputError> {
    // The default options include a parse budget (depth, alias expansion,
    // node count), which protects against "billion laughs" style inputs. They
    // also reject duplicate mapping keys instead of silently keeping one.
    serde_saphyr::from_str::<Value>(text).map_err(|e| {
        let location = e.location();
        InputError::Parse {
            path: source.to_owned(),
            format: Format::Yaml,
            message: strip_yaml_location(&e.without_snippet().to_string()),
            line: location.map(|l| l.line()),
            column: location.map(|l| l.column()),
        }
    })
}

/// `serde-saphyr` messages can carry the location as a prefix
/// ("error: line 2 column 1: <message>") or a suffix ("<message> at line 2,
/// column 1 ..."). Keep just the message; the location is reported separately.
fn strip_yaml_location(message: &str) -> String {
    let mut message = message.strip_prefix("error: ").unwrap_or(message);
    if message.starts_with("line ")
        && let Some((_, rest)) = message.split_once(": ")
    {
        message = rest;
    }
    match message.find(" at line ") {
        Some(i) => message[..i].to_owned(),
        None => message.to_owned(),
    }
}

fn check_version(source: &Path, root: &Value) -> Result<(OpenApiVersion, String), InputError> {
    let path = || source.to_owned();
    let Some(object) = root.as_object() else {
        return Err(InputError::NotAnObject { path: path() });
    };
    let Some(raw) = object.get("openapi") else {
        if object.contains_key("swagger") {
            return Err(InputError::Swagger2 { path: path() });
        }
        return Err(InputError::MissingVersion { path: path() });
    };
    let Some(version) = raw.as_str() else {
        return Err(InputError::VersionNotString { path: path() });
    };
    let parsed = match version.split('.').collect::<Vec<_>>().as_slice() {
        ["3", "0", patch] if is_patch(patch) => Some(OpenApiVersion::V3_0),
        ["3", "1", patch] if is_patch(patch) => Some(OpenApiVersion::V3_1),
        _ => None,
    };
    match parsed {
        Some(v) => Ok((v, version.to_owned())),
        None => Err(InputError::UnsupportedVersion {
            path: path(),
            found: version.to_owned(),
        }),
    }
}

/// A patch component is digits, optionally followed by a pre-release suffix
/// such as `0-rc1`.
fn is_patch(patch: &str) -> bool {
    let digits = patch.split('-').next().unwrap_or("");
    !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(name: &str, text: &str) -> Result<Document, InputError> {
        parse(Path::new(name), text)
    }

    #[test]
    fn json_and_yaml_produce_the_same_value() {
        let json = parse_str(
            "a.json",
            r#"{"openapi":"3.0.3","paths":{"/a":{"get":{"responses":{"200":{"description":"ok"}}}}}}"#,
        )
        .unwrap();
        let yaml = parse_str(
            "a.yaml",
            "openapi: 3.0.3\npaths:\n  /a:\n    get:\n      responses:\n        200:\n          description: ok\n",
        )
        .unwrap();
        assert_eq!(json.format, Format::Json);
        assert_eq!(yaml.format, Format::Yaml);
        assert_eq!(json.root, yaml.root);
    }

    #[test]
    fn detects_format_from_content_without_extension() {
        assert_eq!(
            parse_str("spec", r#"{"openapi":"3.1.0"}"#).unwrap().format,
            Format::Json
        );
        assert_eq!(
            parse_str("spec", "openapi: 3.1.0\n").unwrap().format,
            Format::Yaml
        );
    }

    #[test]
    fn accepts_supported_versions() {
        let v = |s: &str| parse_str("a.yaml", &format!("openapi: '{s}'\n")).map(|d| d.version);
        assert_eq!(v("3.0.0").unwrap(), OpenApiVersion::V3_0);
        assert_eq!(v("3.0.3").unwrap(), OpenApiVersion::V3_0);
        assert_eq!(v("3.1.1").unwrap(), OpenApiVersion::V3_1);
        assert_eq!(v("3.1.0-rc1").unwrap(), OpenApiVersion::V3_1);
    }

    #[test]
    fn rejects_unsupported_versions() {
        for found in ["3.2.0", "2.0", "3.0", "4.0.0", "3.0.x", "3.0."] {
            let err = parse_str("a.yaml", &format!("openapi: '{found}'\n")).unwrap_err();
            assert!(
                matches!(err, InputError::UnsupportedVersion { .. }),
                "{found}: {err}"
            );
        }
    }

    #[test]
    fn unquoted_yaml_version_is_a_number() {
        let err = parse_str("a.yaml", "openapi: 3.0\n").unwrap_err();
        assert!(matches!(err, InputError::VersionNotString { .. }), "{err}");
    }

    #[test]
    fn swagger_2_is_reported_explicitly() {
        let err = parse_str("a.yaml", "swagger: '2.0'\n").unwrap_err();
        assert!(matches!(err, InputError::Swagger2 { .. }), "{err}");
    }

    #[test]
    fn missing_version_and_non_object_root() {
        assert!(matches!(
            parse_str("a.yaml", "info: {}\n").unwrap_err(),
            InputError::MissingVersion { .. }
        ));
        assert!(matches!(
            parse_str("a.yaml", "- 1\n- 2\n").unwrap_err(),
            InputError::NotAnObject { .. }
        ));
    }

    #[test]
    fn json_error_has_location_and_clean_message() {
        let err = parse_str("bad.json", "{\n  \"openapi\": \"3.0.0\",\n}").unwrap_err();
        let InputError::Parse {
            line,
            column,
            ref message,
            ..
        } = err
        else {
            panic!("unexpected error: {err}");
        };
        assert_eq!(line, Some(3));
        assert!(column.is_some());
        assert!(!message.contains(" at line "), "{message}");
        assert!(err.to_string().starts_with("bad.json:3:"), "{err}");
    }

    #[test]
    fn yaml_error_has_location_and_clean_message() {
        let err = parse_str("bad.yaml", "a: [1, 2\nb: 3\n").unwrap_err();
        let InputError::Parse {
            line, ref message, ..
        } = err
        else {
            panic!("unexpected error: {err}");
        };
        assert_eq!(line, Some(2));
        assert!(!message.starts_with("error:"), "{message}");
        assert!(!message.contains(" at line "), "{message}");
        assert!(
            !message.contains('\n'),
            "message should be one line: {message}"
        );
        assert!(err.to_string().starts_with("bad.yaml:2:"), "{err}");
    }

    #[test]
    fn yaml_duplicate_keys_are_rejected() {
        let err = parse_str("dup.yaml", "openapi: 3.0.0\nopenapi: 3.1.0\n").unwrap_err();
        assert!(matches!(err, InputError::Parse { .. }), "{err}");
    }

    #[test]
    fn byte_order_mark_is_ignored() {
        assert!(parse_str("a.json", "\u{feff}{\"openapi\":\"3.0.0\"}").is_ok());
    }

    #[test]
    fn yaml_alias_bombs_are_rejected() {
        // A small "billion laughs" input: each level multiplies the previous one by 9.
        let mut text = String::from("openapi: 3.0.0\na0: &a0 [x, x, x, x, x, x, x, x, x]\n");
        for i in 1..12 {
            let prev = format!("*a{}", i - 1);
            let items = [prev.as_str(); 9].join(", ");
            text.push_str(&format!("a{i}: &a{i} [{items}]\n"));
        }
        let err = parse_str("bomb.yaml", &text).unwrap_err();
        assert!(err.to_string().contains("budget"), "{err}");
    }
}
