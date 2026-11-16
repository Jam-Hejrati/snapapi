//! Schema composition changed. See docs/rules/COMPOSITION_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "COMPOSITION_CHANGED",
    request: Severity::Review,
    response: Severity::Review,
    summary: "Schema composition changed",
    explanation: "A typed client may see a different union or intersection. Full composition semantics require human review.",
    documentation: "docs/rules/COMPOSITION_CHANGED.md",
};
