//! Schema type changed. See docs/rules/SCHEMA_TYPE_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "SCHEMA_TYPE_CHANGED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Schema type changed",
    explanation: "A generated SDK or TypeScript client may use the old type; request validation or response deserialization may fail.",
    documentation: "docs/rules/SCHEMA_TYPE_CHANGED.md",
};
