//! Operation matching and initial added/removed operation findings.

use serde_json::Value;

use crate::finding::Severity;
use crate::model::{ApiModel, Operation, OperationKey};

#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    pub rule_id: &'static str,
    pub severity: Severity,
    pub operation: String,
    pub location: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
    pub summary: String,
    pub explanation: String,
    /// Operation identities affected by the underlying change; enables later grouping.
    pub affected_operations: Vec<String>,
}

pub fn compare(baseline: &ApiModel, candidate: &ApiModel) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (key, old) in &baseline.operations {
        if !candidate.operations.contains_key(key) {
            findings.push(operation_finding(
                "OPERATION_REMOVED",
                Severity::Breaking,
                old,
                None,
            ));
        }
    }
    for (key, new) in &candidate.operations {
        if !baseline.operations.contains_key(key) {
            findings.push(operation_finding(
                "OPERATION_ADDED",
                Severity::Compatible,
                new,
                None,
            ));
        }
    }
    findings.sort_by(|a, b| {
        (a.operation.as_str(), a.rule_id, a.location.as_str()).cmp(&(
            b.operation.as_str(),
            b.rule_id,
            b.location.as_str(),
        ))
    });
    findings
}

fn operation_finding(
    rule: &'static str,
    severity: Severity,
    op: &Operation,
    _other: Option<&Operation>,
) -> Finding {
    let identity = format!("{} {}", op.method.as_str().to_ascii_uppercase(), op.path);
    let (before, after, summary, explanation) = if rule == "OPERATION_ADDED" {
        (
            None,
            Some(op.raw.clone()),
            "Operation added".to_owned(),
            "Existing clients are unaffected; the API now offers an additional operation."
                .to_owned(),
        )
    } else {
        (
            Some(op.raw.clone()),
            None,
            "Operation removed".to_owned(),
            "Clients calling this operation may no longer be able to complete that interaction."
                .to_owned(),
        )
    };
    Finding {
        rule_id: rule,
        severity,
        operation: identity.clone(),
        location: op.location.clone(),
        before,
        after,
        summary,
        explanation,
        affected_operations: vec![identity],
    }
}

pub fn matched_keys(baseline: &ApiModel, candidate: &ApiModel) -> Vec<OperationKey> {
    baseline
        .operations
        .keys()
        .filter(|k| candidate.operations.contains_key(*k))
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
        assert_eq!(f[1].rule_id, "OPERATION_ADDED");
        assert_eq!(f[1].severity, Severity::Compatible);
        assert!(f[1].before.is_none() && f[1].after.is_some());
        assert_eq!(f[1].affected_operations, vec!["POST /b"]);
    }
}
