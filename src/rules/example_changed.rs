//! Example changed. See docs/rules/EXAMPLE_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "EXAMPLE_CHANGED",
    request: Severity::Informational,
    response: Severity::Informational,
    summary: "Example changed",
    explanation: "Examples changed; their values are redacted and do not ordinarily affect a typed client contract.",
    documentation: "docs/rules/EXAMPLE_CHANGED.md",
};
