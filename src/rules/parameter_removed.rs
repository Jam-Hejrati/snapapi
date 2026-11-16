//! Parameter removed. See docs/rules/PARAMETER_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PARAMETER_REMOVED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Parameter removed",
    explanation: "A generated SDK may still send this parameter; its value may be rejected or silently ignored.",
    documentation: "docs/rules/PARAMETER_REMOVED.md",
};
