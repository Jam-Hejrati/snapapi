//! Integration tests for loading documents from disk.

use std::path::PathBuf;

use snapapi::input::{Format, InputError, LoadOptions, OpenApiVersion, load};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/input")
        .join(name)
}

fn load_default(name: &str) -> Result<snapapi::input::Document, InputError> {
    load(&fixture(name), &LoadOptions::default())
}

#[test]
fn loads_yaml_3_0() {
    let doc = load_default("valid-3.0.yaml").unwrap();
    assert_eq!(doc.format, Format::Yaml);
    assert_eq!(doc.version, OpenApiVersion::V3_0);
    assert_eq!(doc.version_string, "3.0.3");
    assert_eq!(doc.root["info"]["title"], "Pets");
}

#[test]
fn loads_json_3_1() {
    let doc = load_default("valid-3.1.json").unwrap();
    assert_eq!(doc.format, Format::Json);
    assert_eq!(doc.version, OpenApiVersion::V3_1);
}

#[test]
fn broken_yaml_reports_path_and_line() {
    let err = load_default("broken.yaml").unwrap_err();
    let msg = err.to_string();
    assert!(
        matches!(
            err,
            InputError::Parse {
                format: Format::Yaml,
                ..
            }
        ),
        "{msg}"
    );
    assert!(msg.contains("broken.yaml:"), "{msg}");
    assert!(msg.contains("invalid YAML"), "{msg}");
}

#[test]
fn broken_json_reports_path_line_and_column() {
    let err = load_default("broken.json").unwrap_err();
    let msg = err.to_string();
    assert!(
        matches!(
            err,
            InputError::Parse {
                format: Format::Json,
                line: Some(3),
                ..
            }
        ),
        "{msg}"
    );
    assert!(msg.contains("broken.json:3:"), "{msg}");
}

#[test]
fn swagger_2_is_rejected_with_hint() {
    let err = load_default("swagger2.yaml").unwrap_err();
    assert!(matches!(err, InputError::Swagger2 { .. }));
    assert!(err.to_string().contains("convert to OpenAPI 3.0 or 3.1"));
}

#[test]
fn missing_file_is_a_read_error() {
    let err = load_default("does-not-exist.yaml").unwrap_err();
    assert!(matches!(err, InputError::Read { .. }), "{err}");
}

#[test]
fn oversized_file_is_rejected() {
    let err = load(&fixture("valid-3.0.yaml"), &LoadOptions { max_bytes: 10 }).unwrap_err();
    assert!(
        matches!(err, InputError::TooLarge { limit: 10, .. }),
        "{err}"
    );
}

#[test]
fn file_exactly_at_limit_is_accepted() {
    let path = fixture("valid-3.0.yaml");
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(load(&path, &LoadOptions { max_bytes: size }).is_ok());
}
