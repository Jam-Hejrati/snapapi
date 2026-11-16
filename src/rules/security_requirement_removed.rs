//! Authentication requirement removed. See docs/rules/SECURITY_REQUIREMENT_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "SECURITY_REQUIREMENT_REMOVED",
    request: Severity::Compatible,
    response: Severity::Compatible,
    summary: "Authentication requirement removed",
    explanation: "Clients can still send authenticated requests; anonymous requests are now permitted.",
    documentation: "docs/rules/SECURITY_REQUIREMENT_REMOVED.md",
};
