//! Response media type removed. See docs/rules/RESPONSE_MEDIA_TYPE_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "RESPONSE_MEDIA_TYPE_REMOVED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Response media type removed",
    explanation: "A client negotiating this content type may lose the response representation it expects.",
    documentation: "docs/rules/RESPONSE_MEDIA_TYPE_REMOVED.md",
};
