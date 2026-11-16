//! Description changed. See docs/rules/DESCRIPTION_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "DESCRIPTION_CHANGED",
    request: Severity::Informational,
    response: Severity::Informational,
    summary: "Description changed",
    explanation: "This documentation change does not ordinarily affect a typed client contract.",
    documentation: "docs/rules/DESCRIPTION_CHANGED.md",
};
