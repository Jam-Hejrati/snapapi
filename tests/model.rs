//! Integration tests for OpenAPI normalization and structural reference handling.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use snapapi::input::{LoadOptions, load, parse};
use snapapi::model::{
    ApiModel, HttpMethod, ModelError, Operation, OperationKey, Presence, normalize,
};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/model")
        .join(name)
}

fn normalize_fixture(name: &str) -> Result<ApiModel, ModelError> {
    let document = load(&fixture(name), &LoadOptions::default()).unwrap();
    normalize(&document)
}

fn normalize_text(text: &str) -> Result<ApiModel, ModelError> {
    let document = parse(Path::new("fixture.yaml"), text).unwrap();
    normalize(&document)
}

fn operation<'a>(model: &'a ApiModel, path: &str, method: HttpMethod) -> &'a Operation {
    model
        .operations
        .get(&OperationKey {
            method,
            path_shape: path.into(),
        })
        .unwrap()
}

#[test]
fn recursive_schema_refs_are_retained_without_expansion() {
    let model = normalize_fixture("recursive-schemas.yaml").unwrap();
    let operation = operation(&model, "/nodes", HttpMethod::Post);

    let Presence::Value(request_body) = &operation.request_body else {
        panic!("expected request body");
    };
    let Presence::Value(request_content) = &request_body.content else {
        panic!("expected request content");
    };
    assert_eq!(
        request_content.pointer("/application~1json/schema/$ref"),
        Some(&json!("#/components/schemas/A"))
    );

    let Presence::Value(responses) = &operation.responses else {
        panic!("expected responses");
    };
    let Presence::Value(response_content) = &responses["200"].content else {
        panic!("expected response content");
    };
    assert_eq!(
        response_content.pointer("/application~1json/schema/$ref"),
        Some(&json!("#/components/schemas/Node"))
    );
}

#[test]
fn structural_refs_and_inherited_parameters_are_normalized() {
    let model = normalize_fixture("structural-refs.yaml").unwrap();
    let operation = operation(&model, "/users", HttpMethod::Post);

    let Presence::Value(parameters) = &operation.parameters else {
        panic!("expected parameters");
    };
    assert_eq!(parameters.len(), 3);

    let header = parameters
        .values()
        .find(|parameter| parameter.location == Presence::Value(json!("header")))
        .unwrap();
    assert_eq!(header.name, Presence::Value(json!("x-trace-id")));
    assert_eq!(header.required, Presence::Value(json!(true)));
    let Presence::Value(schema) = &header.schema else {
        panic!("expected header schema");
    };
    assert_eq!(schema.value["$ref"], "#/components/schemas/TraceId");

    let query_names = parameters
        .values()
        .filter(|parameter| parameter.location == Presence::Value(json!("query")))
        .map(|parameter| match &parameter.name {
            Presence::Value(Value::String(name)) => name.as_str(),
            _ => panic!("expected query parameter name"),
        })
        .collect::<Vec<_>>();
    assert_eq!(query_names, ["Filter", "filter"]);

    let Presence::Value(request_body) = &operation.request_body else {
        panic!("expected request body");
    };
    assert_eq!(request_body.required, Presence::Value(json!(true)));
    let Presence::Value(content) = &request_body.content else {
        panic!("expected request content");
    };
    assert_eq!(
        content.pointer("/application~1json/schema/$ref"),
        Some(&json!("#/components/schemas/User"))
    );

    let Presence::Value(responses) = &operation.responses else {
        panic!("expected responses");
    };
    assert_eq!(
        responses["201"].description,
        Presence::Value(json!("created override"))
    );
}

#[test]
fn openapi_3_0_reference_siblings_do_not_override_the_target() {
    let model = normalize_text(
        "openapi: '3.0.3'\ninfo: {title: t, version: '1'}\npaths:\n  /x:\n    get:\n      responses:\n        '200':\n          $ref: '#/components/responses/Ok'\n          description: ignored\ncomponents:\n  responses:\n    Ok:\n      description: target\n",
    )
    .unwrap();
    let operation = operation(&model, "/x", HttpMethod::Get);
    let Presence::Value(responses) = &operation.responses else {
        panic!("expected responses");
    };
    assert_eq!(
        responses["200"].description,
        Presence::Value(json!("target"))
    );
}

#[test]
fn structural_reference_failures_are_deterministic() {
    assert!(matches!(
        normalize_fixture("remote-ref.yaml").unwrap_err(),
        ModelError::RemoteRef { .. }
    ));
    assert!(matches!(
        normalize_fixture("unresolved-ref.yaml").unwrap_err(),
        ModelError::UnresolvedRef { .. }
    ));
    assert!(matches!(
        normalize_fixture("structural-cycle.yaml").unwrap_err(),
        ModelError::RefCycle { .. }
    ));
    assert!(matches!(
        normalize_fixture("ambiguous-path-ref.yaml").unwrap_err(),
        ModelError::AmbiguousPathItemRef { .. }
    ));

    let encoded = normalize_text(
        "openapi: '3.1.0'\ninfo: {title: t, version: '1'}\npaths:\n  /x:\n    $ref: '#/components/pathItems/Some%20Path'\n",
    )
    .unwrap_err();
    assert!(matches!(encoded, ModelError::UnsupportedLocalRef { .. }));
    assert!(encoded.to_string().contains("percent-encoded fragments"));
}

#[test]
fn escaped_json_pointer_tokens_resolve_structural_refs() {
    let model = normalize_text(
        "openapi: '3.1.0'\ninfo: {title: t, version: '1'}\npaths:\n  /x:\n    $ref: '#/components/pathItems/Some~1Path~0Item'\ncomponents:\n  pathItems:\n    Some/Path~Item:\n      get:\n        responses: {}\n",
    )
    .unwrap();
    assert_eq!(operation(&model, "/x", HttpMethod::Get).path, "/x");
}

#[test]
fn malformed_operations_and_parameters_return_errors() {
    let malformed = normalize_fixture("malformed-operation.yaml").unwrap_err();
    assert!(matches!(malformed, ModelError::InvalidShape { .. }));
    assert!(malformed.to_string().contains("/paths/~1users/get"));

    let invalid_parameter = normalize_text(
        "openapi: '3.0.3'\ninfo: {title: t, version: '1'}\npaths:\n  /x:\n    get:\n      parameters:\n        - name: 12\n          in: query\n      responses: {}\n",
    )
    .unwrap_err();
    assert!(matches!(invalid_parameter, ModelError::InvalidField { .. }));
    assert!(
        invalid_parameter
            .to_string()
            .contains("expected a string at name")
    );
}

#[test]
fn absent_empty_and_null_parameter_states_remain_distinct() {
    let model = normalize_text(
        "openapi: '3.0.3'\ninfo: {title: t, version: '1'}\npaths:\n  /absent:\n    get:\n      responses: {}\n  /empty:\n    get:\n      parameters: []\n      responses: {}\n  /null:\n    get:\n      parameters: null\n      requestBody: null\n      responses: null\n",
    )
    .unwrap();

    assert!(matches!(
        operation(&model, "/absent", HttpMethod::Get).parameters,
        Presence::Absent
    ));
    assert!(matches!(
        operation(&model, "/empty", HttpMethod::Get).parameters,
        Presence::Value(ref parameters) if parameters.is_empty()
    ));
    let null = operation(&model, "/null", HttpMethod::Get);
    assert!(matches!(null.parameters, Presence::Null));
    assert!(matches!(null.request_body, Presence::Null));
    assert!(matches!(null.responses, Presence::Null));
}

#[test]
fn duplicate_path_shapes_are_rejected() {
    let error = normalize_text(
        "openapi: '3.0.3'\ninfo: {title: t, version: '1'}\npaths:\n  /users/{id}:\n    get:\n      responses: {}\n  /users/{name}:\n    get:\n      responses: {}\n",
    )
    .unwrap_err();
    assert!(matches!(error, ModelError::InvalidShape { .. }));
    assert!(error.to_string().contains("duplicate operation path shape"));
}
