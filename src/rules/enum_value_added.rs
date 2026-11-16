//! Enum value added. See docs/rules/ENUM_VALUE_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "ENUM_VALUE_ADDED",
    request: Severity::Compatible,
    response: Severity::Review,
    summary: "Enum value added",
    explanation: "Requests accept additional values. Response clients may use an exhaustive enum switch that lacks the new case.",
    documentation: "docs/rules/ENUM_VALUE_ADDED.md",
};
