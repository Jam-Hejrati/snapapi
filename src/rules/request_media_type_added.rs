//! Request media type added. See docs/rules/REQUEST_MEDIA_TYPE_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "REQUEST_MEDIA_TYPE_ADDED",
    request: Severity::Compatible,
    response: Severity::Compatible,
    summary: "Request media type added",
    explanation: "Existing payload formats remain accepted; clients have an additional content type available.",
    documentation: "docs/rules/REQUEST_MEDIA_TYPE_ADDED.md",
};
