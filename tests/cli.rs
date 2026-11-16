use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_snapapi"))
        .current_dir(root())
        .args(args)
        .output()
        .unwrap()
}
fn rule_paths(rule: &str) -> (String, String) {
    let dir = root().join("tests/fixtures/rules").join(rule);
    (
        dir.join("baseline.json").to_str().unwrap().into(),
        dir.join("candidate.json").to_str().unwrap().into(),
    )
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "snapapi-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str, text: &str) -> String {
        let path = self.0.join(name);
        std::fs::write(&path, text).unwrap();
        path.to_str().unwrap().into()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_exit_codes_come_from_findings_and_policy() {
    let (a, b) = rule_paths("SCHEMA_TYPE_CHANGED");
    for (policy, code) in [
        ("breaking", 1),
        ("review", 1),
        ("any-change", 1),
        ("none", 0),
    ] {
        let output = cli(&["compare", &a, &b, "--format", "json", "--fail-on", policy]);
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["changes"][0]["severity"], "breaking");
        assert_eq!(report["policy_violated"], code == 1);
        assert_eq!(report["schema_version"], 1);
    }
    let (a, b) = rule_paths("RESPONSE_STATUS_ADDED");
    assert_eq!(cli(&["compare", &a, &b]).status.code(), Some(0));
    assert_eq!(
        cli(&["compare", &a, &b, "--fail-on", "review"])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(cli(&["compare"]).status.code(), Some(2));
    assert_eq!(
        cli(&["compare", "absent.json", "absent2.json"])
            .status
            .code(),
        Some(3)
    );
    assert_eq!(cli(&["--help"]).status.code(), Some(0));
    assert_eq!(cli(&["version"]).status.code(), Some(0));
}

#[test]
fn config_is_validated_and_cli_overrides_are_explicit() {
    let temp = Temp::new();
    let (a, b) = rule_paths("SCHEMA_TYPE_CHANGED");
    let config = temp.file(
        "config.yaml",
        "version: 1\npolicy:\n  fail_on: none\noutput:\n  format: json\n",
    );
    let output = cli(&["compare", &a, &b, "--config", &config]);
    assert_eq!(output.status.code(), Some(0));
    assert!(serde_json::from_slice::<Value>(&output.stdout).is_ok());
    assert_eq!(
        cli(&[
            "compare",
            &a,
            &b,
            "--config",
            &config,
            "--fail-on",
            "breaking"
        ])
        .status
        .code(),
        Some(1)
    );
    for text in [
        "version: 2\n",
        "version: 1\nunknown: true\n",
        "version: 1\npolicy:\n  fail_on: typo\n",
        "version: 1\nversion: 1\n",
    ] {
        let bad = temp.file("bad.yaml", text);
        assert_eq!(
            cli(&["compare", &a, &b, "--config", &bad]).status.code(),
            Some(2)
        );
    }
    assert_eq!(
        cli(&["compare", &a, &b, "--max-bytes", "1"]).status.code(),
        Some(3)
    );
}

#[test]
fn reports_preserve_diagnostics_redaction_and_output_errors() {
    let (a, b) = rule_paths("EXAMPLE_CHANGED");
    let text = cli(&["compare", &a, &b, "--verbose"]);
    let markdown = cli(&["compare", &a, &b, "--format", "markdown", "--verbose"]);
    assert!(String::from_utf8_lossy(&text.stdout).contains("redacted"));
    assert!(
        String::from_utf8_lossy(&markdown.stdout)
            .contains("[EXAMPLE_CHANGED](docs/rules/EXAMPLE_CHANGED.md)")
    );
    let path = root().join("tests/fixtures/model/unsupported-webhooks.json");
    let path = path.to_str().unwrap();
    let quiet = cli(&["compare", path, path, "--quiet"]);
    assert!(quiet.stdout.is_empty());
    assert!(String::from_utf8_lossy(&quiet.stderr).contains("webhooks"));
    let json = cli(&["compare", path, path, "--format", "json"]);
    let report: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(report["diagnostics"][0]["feature"], "webhooks");
    let temp = Temp::new();
    let output_path = temp.0.join("report.json");
    assert_eq!(
        cli(&[
            "compare",
            &a,
            &b,
            "--format",
            "json",
            "--output",
            output_path.to_str().unwrap()
        ])
        .status
        .code(),
        Some(0)
    );
    assert!(serde_json::from_str::<Value>(&std::fs::read_to_string(output_path).unwrap()).is_ok());
    assert_eq!(
        cli(&["compare", &a, &b, "--output", temp.0.to_str().unwrap()])
            .status
            .code(),
        Some(4)
    );
    let before = std::fs::read(&a).unwrap();
    assert_eq!(
        cli(&["compare", &a, &b, "--output", &a]).status.code(),
        Some(2)
    );
    assert_eq!(std::fs::read(Path::new(&a)).unwrap(), before);
}
