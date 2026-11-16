//! Operation removed. See docs/rules/OPERATION_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "OPERATION_REMOVED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Operation removed",
    explanation: "A typed client or generated SDK calling this operation may no longer complete that interaction.",
    documentation: "docs/rules/OPERATION_REMOVED.md",
};
