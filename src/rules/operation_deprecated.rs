//! Operation deprecated. See docs/rules/OPERATION_DEPRECATED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "OPERATION_DEPRECATED",
    request: Severity::Informational,
    response: Severity::Informational,
    summary: "Operation deprecated",
    explanation: "The operation remains available; typed clients and SDK consumers should plan their migration.",
    documentation: "docs/rules/OPERATION_DEPRECATED.md",
};
