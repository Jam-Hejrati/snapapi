//! Request media type removed. See docs/rules/REQUEST_MEDIA_TYPE_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "REQUEST_MEDIA_TYPE_REMOVED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Request media type removed",
    explanation: "A client sending this content type may now receive an unsupported-media-type error.",
    documentation: "docs/rules/REQUEST_MEDIA_TYPE_REMOVED.md",
};
