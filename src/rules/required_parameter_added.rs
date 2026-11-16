//! Required parameter added. See docs/rules/REQUIRED_PARAMETER_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "REQUIRED_PARAMETER_ADDED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Required parameter added",
    explanation: "Existing client calls do not include the new required argument and may be rejected.",
    documentation: "docs/rules/REQUIRED_PARAMETER_ADDED.md",
};
