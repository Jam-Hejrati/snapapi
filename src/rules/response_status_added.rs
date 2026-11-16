//! Response status added. See docs/rules/RESPONSE_STATUS_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "RESPONSE_STATUS_ADDED",
    request: Severity::Review,
    response: Severity::Review,
    summary: "Response status added",
    explanation: "Frontend status handling may be exhaustive and may not account for this new response.",
    documentation: "docs/rules/RESPONSE_STATUS_ADDED.md",
};
