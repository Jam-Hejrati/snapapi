//! Security requirement changed. See docs/rules/SECURITY_REQUIREMENT_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "SECURITY_REQUIREMENT_CHANGED",
    request: Severity::Review,
    response: Severity::Review,
    summary: "Security requirement changed",
    explanation: "A frontend client may need different credentials or scopes; the impact depends on authorization policy.",
    documentation: "docs/rules/SECURITY_REQUIREMENT_CHANGED.md",
};
