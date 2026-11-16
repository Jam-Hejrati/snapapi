//! Constraint loosened. See docs/rules/CONSTRAINT_LOOSENED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "CONSTRAINT_LOOSENED",
    request: Severity::Compatible,
    response: Severity::Review,
    summary: "Constraint loosened",
    explanation: "Requests accept a broader range. Response clients may rely on the previous limits and need to handle additional values.",
    documentation: "docs/rules/CONSTRAINT_LOOSENED.md",
};
