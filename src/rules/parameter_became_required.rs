//! Parameter became required. See docs/rules/PARAMETER_BECAME_REQUIRED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PARAMETER_BECAME_REQUIRED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Parameter became required",
    explanation: "A typed client that omits this argument may now receive a request validation error.",
    documentation: "docs/rules/PARAMETER_BECAME_REQUIRED.md",
};
