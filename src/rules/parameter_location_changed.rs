//! Parameter location changed. See docs/rules/PARAMETER_LOCATION_CHANGED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PARAMETER_LOCATION_CHANGED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Parameter location changed",
    explanation: "A generated SDK may send this value in the old location rather than the required new location.",
    documentation: "docs/rules/PARAMETER_LOCATION_CHANGED.md",
};
