//! Property became optional. See docs/rules/PROPERTY_BECAME_OPTIONAL.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PROPERTY_BECAME_OPTIONAL",
    request: Severity::Compatible,
    response: Severity::Breaking,
    summary: "Property became optional",
    explanation: "Requests: clients may omit the field. Responses: clients typed against a required field may now encounter its absence.",
    documentation: "docs/rules/PROPERTY_BECAME_OPTIONAL.md",
};
