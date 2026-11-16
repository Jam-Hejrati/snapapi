//! Schema became nullable. See docs/rules/BECAME_NULLABLE.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "BECAME_NULLABLE",
    request: Severity::Compatible,
    response: Severity::Breaking,
    summary: "Schema became nullable",
    explanation: "Requests may now contain null. A response client typed against a non-null value may now receive null.",
    documentation: "docs/rules/BECAME_NULLABLE.md",
};
