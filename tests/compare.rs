use serde_json::{Value, json};
use snapapi::finding::{Direction, Severity};
use snapapi::{compare, input, model, rules};
use std::path::{Path, PathBuf};

fn normalize(path: &Path) -> model::ApiModel {
    model::normalize(&input::load(path, &input::LoadOptions::default()).unwrap()).unwrap()
}

#[test]
fn every_rule_has_a_document_and_a_passing_fixture_pair() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for rule in rules::ALL {
        assert!(
            root.join(rule.documentation).is_file(),
            "missing documentation: {}",
            rule.id
        );
        let dir = root.join("tests/fixtures/rules").join(rule.id);
        let a = normalize(&dir.join("baseline.json"));
        let b = normalize(&dir.join("candidate.json"));
        let findings = compare::compare(&a, &b);
        let actual: Vec<_> = findings
            .iter()
            .map(|f| json!({"rule_id":f.rule_id,"severity":f.severity.as_str()}))
            .collect();
        let expected: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join("expected.json")).unwrap())
                .unwrap();
        assert_eq!(json!(actual), expected, "fixture: {}", rule.id);
        assert!(
            findings
                .iter()
                .all(|f| !f.explanation.is_empty() && f.documentation == rule.documentation)
        );
    }
}

#[test]
fn component_changes_are_detected_grouped_and_cycle_safe() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/regressions/shared-schema");
    let a = normalize(&dir.join("baseline.json"));
    let b = normalize(&dir.join("candidate.json"));
    let findings = compare::compare(&a, &b);
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "SCHEMA_TYPE_CHANGED");
    assert_eq!(findings[0].location, "/components/schemas/User/type");
    assert_eq!(
        findings[0].affected_operations,
        ["GET /teams", "GET /users"]
    );
    for _ in 0..10 {
        assert_eq!(compare::compare(&a, &b), findings);
    }
}

fn schema_comparison(old: Value, new: Value, direction: Direction) -> Vec<compare::Finding> {
    let document = |schema| {
        let contract = json!({"content":{"application/json":{"schema":schema}}});
        let operation = if direction == Direction::Request {
            json!({"requestBody":contract,"responses":{"200":{"description":"ok"}}})
        } else {
            json!({"responses":{"200":contract}})
        };
        let doc = json!({"openapi":"3.1.0","paths":{"/x":{"post":operation}}});
        model::normalize(&input::parse(Path::new("schema.json"), &doc.to_string()).unwrap())
            .unwrap()
    };
    compare::compare(&document(old), &document(new))
}

#[test]
fn schema_rules_follow_request_and_response_directions() {
    for (old, new, rule, request, response) in [
        (
            json!({"type":"string","enum":["a","b"]}),
            json!({"type":"string","enum":["a"]}),
            "ENUM_VALUE_REMOVED",
            Severity::Breaking,
            Severity::Compatible,
        ),
        (
            json!({"type":"string","enum":["a"]}),
            json!({"type":"string","enum":["a","b"]}),
            "ENUM_VALUE_ADDED",
            Severity::Compatible,
            Severity::Review,
        ),
        (
            json!({"type":"string"}),
            json!({"type":["string","null"]}),
            "BECAME_NULLABLE",
            Severity::Compatible,
            Severity::Breaking,
        ),
        (
            json!({"type":"string","minLength":2}),
            json!({"type":"string","minLength":3}),
            "CONSTRAINT_TIGHTENED",
            Severity::Breaking,
            Severity::Compatible,
        ),
    ] {
        for (direction, severity) in [
            (Direction::Request, request),
            (Direction::Response, response),
        ] {
            let findings = schema_comparison(old.clone(), new.clone(), direction);
            assert_eq!(findings.len(), 1);
            assert_eq!(findings[0].rule_id, rule);
            assert_eq!(findings[0].severity, severity);
        }
    }
}

#[test]
fn null_empty_and_reordering_do_not_hide_or_invent_schema_changes() {
    let same = json!({"type":["string","integer"],"enum":["a","b"]});
    assert!(
        schema_comparison(
            same,
            json!({"type":["integer","string"],"enum":["b","a"]}),
            Direction::Request
        )
        .is_empty()
    );
    assert!(
        schema_comparison(
            json!({"type":"string"}),
            json!({"type":"string","minLength":0}),
            Direction::Request
        )
        .is_empty()
    );
    let null = schema_comparison(Value::Null, json!({}), Direction::Request);
    assert_eq!(null[0].rule_id, "SCHEMA_CHANGED");
    assert_eq!(null[0].before, Some(Value::Null));
    let union = schema_comparison(
        json!({"type":"string"}),
        json!({"anyOf":[{"type":"string"},{"type":"null"}]}),
        Direction::Response,
    );
    assert_eq!(union.len(), 1);
    assert_eq!(union[0].rule_id, "BECAME_NULLABLE");
}

#[test]
fn examples_are_redacted_in_all_findings() {
    let findings = schema_comparison(
        json!({"type":"string","example":"secret-old"}),
        json!({"type":"string","example":"secret-new"}),
        Direction::Response,
    );
    let rendered = serde_json::to_string(&findings).unwrap();
    assert!(!rendered.contains("secret-old") && !rendered.contains("secret-new"));
    assert!(rendered.contains("redacted"));
}

#[test]
fn integer_boundaries_and_property_visibility_are_direction_aware() {
    let findings = schema_comparison(
        json!({"type":"string","maxLength":9007199254740993_u64}),
        json!({"type":"string","maxLength":9007199254740992_u64}),
        Direction::Request,
    );
    assert_eq!(findings[0].rule_id, "CONSTRAINT_TIGHTENED");
    let before = json!({"type":"object","properties":{}});
    let readonly = json!({"type":"object","properties":{"id":{"type":"string","readOnly":true}},"required":["id"]});
    assert!(schema_comparison(before.clone(), readonly, Direction::Request).is_empty());
    let writeonly = json!({"type":"object","properties":{"password":{"type":"string","writeOnly":true}},"required":["password"]});
    assert!(schema_comparison(before, writeonly, Direction::Response).is_empty());
    let null_type = schema_comparison(json!({}), json!({"type":null}), Direction::Request);
    assert_eq!(null_type[0].rule_id, "SCHEMA_CHANGED");
    assert_eq!(null_type[0].severity, Severity::Review);
}

#[test]
fn schema_reference_siblings_do_not_produce_definite_breaking_claims() {
    let make = |schema: Value| {
        let doc = json!({"openapi":"3.1.0","components":{"schemas":{"Text":{"type":"string"}}},"paths":{"/x":{"get":{"responses":{"200":{"content":{"application/json":{"schema":schema}}}}}}}});
        model::normalize(&input::parse(Path::new("reference.json"), &doc.to_string()).unwrap())
            .unwrap()
    };
    let a = make(json!({"$ref":"#/components/schemas/Text","maxLength":10}));
    let b = make(json!({"type":"string","maxLength":10}));
    let findings = compare::compare(&a, &b);
    assert!(!findings.is_empty());
    assert!(findings.iter().all(|f| f.severity == Severity::Review));
    assert!(
        a.diagnostics
            .iter()
            .any(|d| d.feature == "schema $ref siblings")
    );
}

#[test]
fn parameter_renames_and_security_order_are_normalized() {
    let make = |path: &str, name: &str, scopes: Value| {
        let doc = json!({"openapi":"3.1.0","security":[{"auth":scopes}],"paths":{path:{"get":{"parameters":[{"name":name,"in":"path","required":true,"schema":{"type":"string"}}],"responses":{}}}}});
        model::normalize(&input::parse(Path::new("parameters.json"), &doc.to_string()).unwrap())
            .unwrap()
    };
    let a = make("/users/{id}", "id", json!(["read", "write"]));
    let b = make("/users/{userId}", "userId", json!(["write", "read"]));
    assert!(compare::compare(&a, &b).is_empty());
}

#[test]
fn referenced_readonly_properties_are_not_required_request_arguments() {
    let doc = |properties: Value, required: Value| {
        let document = json!({"openapi":"3.1.0","components":{"schemas":{"Id":{"type":"string","readOnly":true}}},"paths":{"/x":{"post":{"requestBody":{"content":{"application/json":{"schema":{"type":"object","properties":properties,"required":required}}}},"responses":{}}}}});
        model::normalize(&input::parse(Path::new("readonly.json"), &document.to_string()).unwrap())
            .unwrap()
    };
    let old = doc(json!({}), json!([]));
    let new = doc(
        json!({"id":{"$ref":"#/components/schemas/Id"}}),
        json!(["id"]),
    );
    assert!(compare::compare(&old, &new).is_empty());
}
