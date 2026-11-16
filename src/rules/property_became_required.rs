//! Property became required. See docs/rules/PROPERTY_BECAME_REQUIRED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PROPERTY_BECAME_REQUIRED",
    request: Severity::Breaking,
    response: Severity::Compatible,
    summary: "Property became required",
    explanation: "Requests: existing typed clients may omit this field. Responses: clients now receive the field consistently.",
    documentation: "docs/rules/PROPERTY_BECAME_REQUIRED.md",
};
