//! Enum value removed. See docs/rules/ENUM_VALUE_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "ENUM_VALUE_REMOVED",
    request: Severity::Breaking,
    response: Severity::Compatible,
    summary: "Enum value removed",
    explanation: "Requests: a value sent by an existing client may be rejected. Responses: the set of possible values becomes smaller.",
    documentation: "docs/rules/ENUM_VALUE_REMOVED.md",
};
