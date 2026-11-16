//! Schema semantics changed. See docs/rules/SCHEMA_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "SCHEMA_CHANGED",
    request: Severity::Review,
    response: Severity::Review,
    summary: "Schema semantics changed",
    explanation: "The change involves absent, null, boolean schemas, reference siblings, or a constraint whose impact cannot be safely inferred.",
    documentation: "docs/rules/SCHEMA_CHANGED.md",
};
