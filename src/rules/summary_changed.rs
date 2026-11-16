//! Summary changed. See docs/rules/SUMMARY_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "SUMMARY_CHANGED",
    request: Severity::Informational,
    response: Severity::Informational,
    summary: "Summary changed",
    explanation: "This documentation change does not ordinarily affect a typed client contract.",
    documentation: "docs/rules/SUMMARY_CHANGED.md",
};
