//! Authentication required. See docs/rules/SECURITY_REQUIREMENT_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "SECURITY_REQUIREMENT_ADDED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Authentication required",
    explanation: "Previously anonymous requests from a frontend client may now be rejected.",
    documentation: "docs/rules/SECURITY_REQUIREMENT_ADDED.md",
};
