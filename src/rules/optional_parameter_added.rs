//! Optional parameter added. See docs/rules/OPTIONAL_PARAMETER_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "OPTIONAL_PARAMETER_ADDED",
    request: Severity::Compatible,
    response: Severity::Compatible,
    summary: "Optional parameter added",
    explanation: "Existing client calls remain valid without the new optional argument.",
    documentation: "docs/rules/OPTIONAL_PARAMETER_ADDED.md",
};
