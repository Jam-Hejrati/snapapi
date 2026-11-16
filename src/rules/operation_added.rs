//! Operation added. See docs/rules/OPERATION_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "OPERATION_ADDED",
    request: Severity::Compatible,
    response: Severity::Compatible,
    summary: "Operation added",
    explanation: "Existing clients keep their operations; a generated SDK can expose this additional operation.",
    documentation: "docs/rules/OPERATION_ADDED.md",
};
