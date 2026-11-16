//! Constraint tightened. See docs/rules/CONSTRAINT_TIGHTENED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "CONSTRAINT_TIGHTENED",
    request: Severity::Breaking,
    response: Severity::Compatible,
    summary: "Constraint tightened",
    explanation: "Requests previously accepted from a frontend may now fail validation. Response values stay within narrower limits.",
    documentation: "docs/rules/CONSTRAINT_TIGHTENED.md",
};
