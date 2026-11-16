//! Parameter became optional. See docs/rules/PARAMETER_BECAME_OPTIONAL.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PARAMETER_BECAME_OPTIONAL",
    request: Severity::Compatible,
    response: Severity::Compatible,
    summary: "Parameter became optional",
    explanation: "Client requests remain valid; a generated SDK can now allow the argument to be omitted.",
    documentation: "docs/rules/PARAMETER_BECAME_OPTIONAL.md",
};
