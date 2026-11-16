//! Property added. See docs/rules/PROPERTY_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PROPERTY_ADDED",
    request: Severity::Compatible,
    response: Severity::Compatible,
    summary: "Property added",
    explanation: "Existing clients are expected to tolerate this optional request property or ignore this additional response field.",
    documentation: "docs/rules/PROPERTY_ADDED.md",
};
