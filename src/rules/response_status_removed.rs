//! Response status removed. See docs/rules/RESPONSE_STATUS_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "RESPONSE_STATUS_REMOVED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Response status removed",
    explanation: "A typed client may depend on handling this response status and its payload.",
    documentation: "docs/rules/RESPONSE_STATUS_REMOVED.md",
};
