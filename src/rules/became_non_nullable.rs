//! Schema became non-nullable. See docs/rules/BECAME_NON_NULLABLE.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "BECAME_NON_NULLABLE",
    request: Severity::Breaking,
    response: Severity::Compatible,
    summary: "Schema became non-nullable",
    explanation: "A client sending null may now be rejected. Response clients receive a stronger non-null guarantee.",
    documentation: "docs/rules/BECAME_NON_NULLABLE.md",
};
