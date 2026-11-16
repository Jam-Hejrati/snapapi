//! Deterministic, direction-aware comparison of operations and their contracts.

use crate::finding::{Direction, Severity};
use crate::model::{ApiModel, Operation, OperationKey, Parameter, Presence, escape};
use crate::rules::{self, Rule};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Finding {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub direction: Option<Direction>,
    pub operation: String,
    pub location: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<Value>,
    pub summary: String,
    pub explanation: String,
    pub documentation: &'static str,
    /// Stable identity of the underlying change, separate from consumer impacts.
    pub change_id: String,
    pub affected_operations: Vec<String>,
}

pub(crate) fn finding(
    rule: Rule,
    direction: Option<Direction>,
    operation: &Operation,
    location: &str,
    before: Option<Value>,
    after: Option<Value>,
) -> Finding {
    let identity = format!(
        "{} {}",
        operation.method.as_str().to_ascii_uppercase(),
        operation.path
    );
    Finding {
        rule_id: rule.id,
        severity: rule.severity(direction.unwrap_or(Direction::Request)),
        direction,
        operation: identity.clone(),
        location: location.into(),
        before: before.map(safe_value),
        after: after.map(safe_value),
        summary: rule.summary.into(),
        explanation: rule.explanation.into(),
        documentation: rule.documentation,
        change_id: format!("{}:{direction:?}:{location}", rule.id),
        affected_operations: vec![identity],
    }
}

fn safe_value(mut value: Value) -> Value {
    fn redact(value: &mut Value) {
        match value {
            Value::Object(object) => {
                for (key, value) in object {
                    if matches!(key.as_str(), "example" | "examples" | "default") {
                        *value = json!("<redacted>");
                    } else {
                        redact(value);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    redact(value);
                }
            }
            _ => {}
        }
    }
    redact(&mut value);
    if value.to_string().len() > 2048 {
        json!("<value omitted: exceeds 2048 bytes>")
    } else {
        value
    }
}

pub fn compare(baseline: &ApiModel, candidate: &ApiModel) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (key, old) in &baseline.operations {
        match candidate.operations.get(key) {
            None => findings.push(finding(
                rules::OPERATION_REMOVED,
                None,
                old,
                &old.location,
                Some(json!({"method": old.method.as_str(), "path": old.path})),
                None,
            )),
            Some(new) => compare_operation(baseline, candidate, old, new, &mut findings),
        }
    }
    for (key, new) in &candidate.operations {
        if !baseline.operations.contains_key(key) {
            findings.push(finding(
                rules::OPERATION_ADDED,
                None,
                new,
                &new.location,
                None,
                Some(json!({"method": new.method.as_str(), "path": new.path})),
            ));
        }
    }
    let mut grouped = BTreeMap::<String, Finding>::new();
    for item in findings {
        let key = format!(
            "{}:{}:{:?}:{:?}",
            item.change_id,
            item.severity.as_str(),
            item.before,
            item.after
        );
        if let Some(existing) = grouped.get_mut(&key) {
            existing
                .affected_operations
                .extend(item.affected_operations);
        } else {
            grouped.insert(key, item);
        }
    }
    let mut findings: Vec<_> = grouped.into_values().collect();
    for item in &mut findings {
        item.affected_operations.sort();
        item.affected_operations.dedup();
        item.operation = item.affected_operations[0].clone();
    }
    findings.sort_by(|a, b| {
        (&a.operation, a.rule_id, &a.location).cmp(&(&b.operation, b.rule_id, &b.location))
    });
    findings
}

fn compare_operation(
    baseline: &ApiModel,
    candidate: &ApiModel,
    old: &Operation,
    new: &Operation,
    output: &mut Vec<Finding>,
) {
    metadata(&old.raw, &new.raw, new, &new.location, output);
    if old.raw.get("deprecated").and_then(Value::as_bool) != Some(true)
        && new.raw.get("deprecated").and_then(Value::as_bool) == Some(true)
    {
        output.push(finding(
            rules::OPERATION_DEPRECATED,
            None,
            new,
            &format!("{}/deprecated", new.location),
            Some(json!(false)),
            Some(json!(true)),
        ));
    }
    let security_old = presence(&old.security);
    let security_new = presence(&new.security);
    let normalized = |v: &Option<Value>| {
        let mut entries: Vec<_> = v
            .as_ref()
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|entry| {
                        let mut entry = entry.clone();
                        if let Some(object) = entry.as_object_mut() {
                            for scopes in object.values_mut() {
                                if let Some(values) = scopes.as_array_mut() {
                                    values.sort_by_key(Value::to_string);
                                    values.dedup();
                                }
                            }
                        }
                        entry.to_string()
                    })
                    .collect()
            })
            .unwrap_or_default();
        entries.sort();
        entries.dedup();
        if entries == ["{}"] {
            entries.clear();
        }
        entries
    };
    if normalized(&security_old) != normalized(&security_new)
        || matches!(old.security, Presence::Null) != matches!(new.security, Presence::Null)
    {
        let anonymous = |v: &Option<Value>| {
            v.as_ref().is_none_or(|v| {
                v.as_array().is_some_and(|a| {
                    a.is_empty()
                        || a.iter()
                            .any(|r| r.as_object().is_some_and(|o| o.is_empty()))
                })
            })
        };
        let rule = if anonymous(&security_old)
            && !anonymous(&security_new)
            && !matches!(new.security, Presence::Null)
        {
            rules::SECURITY_REQUIREMENT_ADDED
        } else if !anonymous(&security_old) && anonymous(&security_new) {
            rules::SECURITY_REQUIREMENT_REMOVED
        } else {
            rules::SECURITY_REQUIREMENT_CHANGED
        };
        output.push(finding(
            rule,
            None,
            new,
            &format!("{}/security", new.location),
            security_old,
            security_new,
        ));
    }
    compare_parameters(baseline, candidate, old, new, output);
    let old_body = match &old.request_body {
        Presence::Value(v) => Some(v),
        _ => None,
    };
    let new_body = match &new.request_body {
        Presence::Value(v) => Some(v),
        _ => None,
    };
    let old_required = old_body.and_then(|b| presence(&b.required));
    let new_required = new_body.and_then(|b| presence(&b.required));
    requiredness(
        old_required,
        new_required,
        rules::REQUEST_BODY_BECAME_REQUIRED,
        rules::REQUEST_BODY_BECAME_OPTIONAL,
        new,
        &format!("{}/requestBody/required", new.location),
        output,
    );
    if matches!(old.request_body, Presence::Null) != matches!(new.request_body, Presence::Null) {
        output.push(finding(
            rules::SCHEMA_CHANGED,
            Some(Direction::Request),
            new,
            &format!("{}/requestBody", new.location),
            matches!(old.request_body, Presence::Null).then_some(Value::Null),
            matches!(new.request_body, Presence::Null).then_some(Value::Null),
        ));
    }
    compare_content(
        baseline,
        candidate,
        old_body.and_then(|b| presence(&b.content)),
        new_body.and_then(|b| presence(&b.content)),
        &format!("{}/requestBody/content", old.location),
        &format!("{}/requestBody/content", new.location),
        new,
        Direction::Request,
        output,
    );
    compare_responses(baseline, candidate, old, new, output);
}

pub(crate) fn metadata(
    old: &Value,
    new: &Value,
    operation: &Operation,
    location: &str,
    output: &mut Vec<Finding>,
) {
    for (field, rule) in [
        ("description", rules::DESCRIPTION_CHANGED),
        ("summary", rules::SUMMARY_CHANGED),
        ("example", rules::EXAMPLE_CHANGED),
        ("examples", rules::EXAMPLE_CHANGED),
    ] {
        if old.get(field) != new.get(field) {
            let value = |v: Option<&Value>| {
                if field.starts_with("example") {
                    v.map(|_| json!("<redacted>"))
                } else {
                    v.cloned()
                }
            };
            output.push(finding(
                rule,
                None,
                operation,
                &format!("{location}/{field}"),
                value(old.get(field)),
                value(new.get(field)),
            ));
        }
    }
}

pub(crate) fn presence(value: &Presence<Value>) -> Option<Value> {
    match value {
        Presence::Absent => None,
        Presence::Null => Some(Value::Null),
        Presence::Value(v) => Some(v.clone()),
    }
}

fn requiredness(
    old: Option<Value>,
    new: Option<Value>,
    required: Rule,
    optional: Rule,
    operation: &Operation,
    location: &str,
    output: &mut Vec<Finding>,
) {
    if old.as_ref().is_some_and(Value::is_null) || new.as_ref().is_some_and(Value::is_null) {
        if old != new {
            output.push(finding(
                rules::SCHEMA_CHANGED,
                Some(Direction::Request),
                operation,
                location,
                old,
                new,
            ));
        }
    } else {
        let a = old.as_ref().and_then(Value::as_bool).unwrap_or(false);
        let b = new.as_ref().and_then(Value::as_bool).unwrap_or(false);
        if a != b {
            output.push(finding(
                if b { required } else { optional },
                Some(Direction::Request),
                operation,
                location,
                Some(json!(a)),
                Some(json!(b)),
            ));
        }
    }
}

fn parameter_map(operation: &Operation) -> BTreeMap<String, &Parameter> {
    let Presence::Value(parameters) = &operation.parameters else {
        return BTreeMap::new();
    };
    parameters
        .iter()
        .map(|(key, parameter)| {
            let key = if parameter.location == Presence::Value(json!("path")) {
                let name = presence(&parameter.name)
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default();
                let tokens: Vec<_> = operation
                    .path
                    .split('{')
                    .skip(1)
                    .filter_map(|p| p.split_once('}').map(|(name, _)| name))
                    .collect();
                tokens
                    .iter()
                    .position(|token| *token == name)
                    .map(|n| format!("path:{n}"))
                    .unwrap_or_else(|| key.clone())
            } else {
                key.clone()
            };
            (key, parameter)
        })
        .collect()
}

fn compare_parameters(
    baseline: &ApiModel,
    candidate: &ApiModel,
    old: &Operation,
    new: &Operation,
    output: &mut Vec<Finding>,
) {
    if matches!(old.parameters, Presence::Null) || matches!(new.parameters, Presence::Null) {
        if matches!(old.parameters, Presence::Null) != matches!(new.parameters, Presence::Null) {
            output.push(finding(
                rules::SCHEMA_CHANGED,
                Some(Direction::Request),
                new,
                &format!("{}/parameters", new.location),
                matches!(old.parameters, Presence::Null).then_some(Value::Null),
                matches!(new.parameters, Presence::Null).then_some(Value::Null),
            ));
        }
        return;
    }
    let before = parameter_map(old);
    let after = parameter_map(new);
    let mut relocated = BTreeSet::new();
    for (key, a) in &before {
        if let Some(b) = after.get(key) {
            requiredness(
                presence(&a.required),
                presence(&b.required),
                rules::PARAMETER_BECAME_REQUIRED,
                rules::PARAMETER_BECAME_OPTIONAL,
                new,
                &format!("{}/required", b.source_location),
                output,
            );
            let schema = |p: &Parameter| match &p.schema {
                Presence::Absent => None,
                Presence::Null => Some(Value::Null),
                Presence::Value(s) => Some(s.value.clone()),
            };
            compare_schema(
                baseline,
                candidate,
                schema(a),
                schema(b),
                &format!("{}/schema", a.source_location),
                &format!("{}/schema", b.source_location),
                new,
                Direction::Request,
                output,
            );
            compare_content(
                baseline,
                candidate,
                presence(&a.content),
                presence(&b.content),
                &format!("{}/content", a.source_location),
                &format!("{}/content", b.source_location),
                new,
                Direction::Request,
                output,
            );
        } else {
            let moved: Vec<_> = after
                .iter()
                .filter(|(new_key, b)| {
                    !before.contains_key(*new_key) && a.name == b.name && a.location != b.location
                })
                .collect();
            if moved.len() == 1 {
                let (new_key, b) = moved[0];
                relocated.insert(new_key.clone());
                output.push(finding(
                    rules::PARAMETER_LOCATION_CHANGED,
                    Some(Direction::Request),
                    new,
                    &b.source_location,
                    presence(&a.location),
                    presence(&b.location),
                ));
            } else {
                output.push(finding(
                    rules::PARAMETER_REMOVED,
                    Some(Direction::Request),
                    old,
                    &a.source_location,
                    presence(&a.name),
                    None,
                ));
            }
        }
    }
    for (key, b) in &after {
        if !before.contains_key(key) && !relocated.contains(key) {
            let required = presence(&b.required) == Some(json!(true))
                || b.location == Presence::Value(json!("path"));
            output.push(finding(
                if required {
                    rules::REQUIRED_PARAMETER_ADDED
                } else {
                    rules::OPTIONAL_PARAMETER_ADDED
                },
                Some(Direction::Request),
                new,
                &b.source_location,
                None,
                presence(&b.name),
            ));
        }
    }
}

fn compare_responses(
    baseline: &ApiModel,
    candidate: &ApiModel,
    old: &Operation,
    new: &Operation,
    output: &mut Vec<Finding>,
) {
    if matches!(old.responses, Presence::Null) || matches!(new.responses, Presence::Null) {
        if matches!(old.responses, Presence::Null) != matches!(new.responses, Presence::Null) {
            output.push(finding(
                rules::SCHEMA_CHANGED,
                Some(Direction::Response),
                new,
                &format!("{}/responses", new.location),
                matches!(old.responses, Presence::Null).then_some(Value::Null),
                matches!(new.responses, Presence::Null).then_some(Value::Null),
            ));
        }
        return;
    }
    let empty = BTreeMap::new();
    let before = match &old.responses {
        Presence::Value(v) => v,
        _ => &empty,
    };
    let after = match &new.responses {
        Presence::Value(v) => v,
        _ => &empty,
    };
    for (code, a) in before {
        let old_location = format!("{}/responses/{}", old.location, escape(code));
        let new_location = format!("{}/responses/{}", new.location, escape(code));
        if let Some(b) = after.get(code) {
            metadata(&a.raw, &b.raw, new, &new_location, output);
            compare_content(
                baseline,
                candidate,
                presence(&a.content),
                presence(&b.content),
                &format!("{old_location}/content"),
                &format!("{new_location}/content"),
                new,
                Direction::Response,
                output,
            );
        } else {
            output.push(finding(
                rules::RESPONSE_STATUS_REMOVED,
                Some(Direction::Response),
                old,
                &old_location,
                Some(json!(code)),
                None,
            ));
        }
    }
    for code in after.keys() {
        if !before.contains_key(code) {
            output.push(finding(
                rules::RESPONSE_STATUS_ADDED,
                Some(Direction::Response),
                new,
                &format!("{}/responses/{}", new.location, escape(code)),
                None,
                Some(json!(code)),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compare_content(
    baseline: &ApiModel,
    candidate: &ApiModel,
    old: Option<Value>,
    new: Option<Value>,
    old_location: &str,
    new_location: &str,
    operation: &Operation,
    direction: Direction,
    output: &mut Vec<Finding>,
) {
    if old.as_ref().is_some_and(Value::is_null) || new.as_ref().is_some_and(Value::is_null) {
        if old != new {
            output.push(finding(
                rules::SCHEMA_CHANGED,
                Some(direction),
                operation,
                new_location,
                old,
                new,
            ));
        }
        return;
    }
    let empty = serde_json::Map::new();
    let before = old.as_ref().and_then(Value::as_object).unwrap_or(&empty);
    let after = new.as_ref().and_then(Value::as_object).unwrap_or(&empty);
    for (media, a) in before {
        if media.starts_with("x-") {
            continue;
        }
        let loc = format!("{new_location}/{}", escape(media));
        if let Some(b) = after.get(media) {
            metadata(a, b, operation, &loc, output);
            compare_schema(
                baseline,
                candidate,
                a.get("schema").cloned(),
                b.get("schema").cloned(),
                &format!("{old_location}/{}/schema", escape(media)),
                &format!("{loc}/schema"),
                operation,
                direction,
                output,
            );
        } else {
            output.push(finding(
                if direction == Direction::Request {
                    rules::REQUEST_MEDIA_TYPE_REMOVED
                } else {
                    rules::RESPONSE_MEDIA_TYPE_REMOVED
                },
                Some(direction),
                operation,
                &loc,
                Some(json!(media)),
                None,
            ));
        }
    }
    for media in after.keys() {
        if !media.starts_with("x-") && !before.contains_key(media) {
            output.push(finding(
                if direction == Direction::Request {
                    rules::REQUEST_MEDIA_TYPE_ADDED
                } else {
                    rules::RESPONSE_MEDIA_TYPE_ADDED
                },
                Some(direction),
                operation,
                &format!("{new_location}/{}", escape(media)),
                None,
                Some(json!(media)),
            ));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compare_schema(
    baseline: &ApiModel,
    candidate: &ApiModel,
    old: Option<Value>,
    new: Option<Value>,
    old_location: &str,
    new_location: &str,
    operation: &Operation,
    direction: Direction,
    output: &mut Vec<Finding>,
) {
    match (old, new) {
        (Some(a), Some(b)) => output.extend(crate::schema::compare(
            baseline,
            candidate,
            &a,
            &b,
            old_location,
            new_location,
            operation,
            direction,
        )),
        (None, None) => {}
        (a, b) => output.push(finding(
            rules::SCHEMA_CHANGED,
            Some(direction),
            operation,
            new_location,
            a,
            b,
        )),
    }
}

pub fn matched_keys(baseline: &ApiModel, candidate: &ApiModel) -> Vec<OperationKey> {
    baseline
        .operations
        .keys()
        .filter(|key| candidate.operations.contains_key(*key))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{input, model};
    use std::path::Path;
    fn model_from(text: &str) -> ApiModel {
        model::normalize(&input::parse(Path::new("fixture.yaml"), text).unwrap()).unwrap()
    }
    fn spec(paths: &str) -> String {
        format!("openapi: '3.0.3'\ninfo: {{title: t, version: '1'}}\npaths:\n{paths}")
    }
    #[test]
    fn matching_ignores_parameter_name_but_not_method_or_literal_path() {
        let a = model_from(&spec(
            "  /users/{id}:\n    get:\n      responses:\n        '200':\n          description: ok\n",
        ));
        let b = model_from(&spec(
            "  /users/{userId}:\n    get:\n      responses:\n        '200':\n          description: ok\n",
        ));
        assert!(compare(&a, &b).is_empty());
        let c = model_from(&spec(
            "  /users/{userId}:\n    post:\n      responses:\n        '200':\n          description: ok\n",
        ));
        let f = compare(&a, &c);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].rule_id, "OPERATION_REMOVED");
        assert_eq!(f[1].rule_id, "OPERATION_ADDED");
    }
    #[test]
    fn additions_and_removals_include_rule_data_and_are_sorted() {
        let a = model_from(&spec("  /z:\n    get:\n      responses: {}\n"));
        let b = model_from(&spec("  /b:\n    post:\n      responses: {}\n"));
        let f = compare(&a, &b);
        assert_eq!(f.len(), 2);
        assert_eq!(f[0].rule_id, "OPERATION_REMOVED");
        assert_eq!(f[0].severity, Severity::Breaking);
        assert!(f[0].before.is_some() && f[0].after.is_none());
        assert_eq!(f[0].before, Some(json!({"method": "get", "path": "/z"})));
        assert_eq!(f[1].rule_id, "OPERATION_ADDED");
        assert_eq!(f[1].severity, Severity::Compatible);
        assert!(f[1].before.is_none() && f[1].after.is_some());
        assert_eq!(f[1].after, Some(json!({"method": "post", "path": "/b"})));
        assert_eq!(f[1].affected_operations, vec!["POST /b"]);
    }
}
