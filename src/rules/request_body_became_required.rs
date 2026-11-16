//! Request body became required. See docs/rules/REQUEST_BODY_BECAME_REQUIRED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "REQUEST_BODY_BECAME_REQUIRED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Request body became required",
    explanation: "Existing frontend calls without a body may now be rejected.",
    documentation: "docs/rules/REQUEST_BODY_BECAME_REQUIRED.md",
};
