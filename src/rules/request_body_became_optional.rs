//! Request body became optional. See docs/rules/REQUEST_BODY_BECAME_OPTIONAL.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "REQUEST_BODY_BECAME_OPTIONAL",
    request: Severity::Compatible,
    response: Severity::Compatible,
    summary: "Request body became optional",
    explanation: "Existing frontend calls remain valid; callers may now omit the body.",
    documentation: "docs/rules/REQUEST_BODY_BECAME_OPTIONAL.md",
};
