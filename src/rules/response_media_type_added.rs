//! Response media type added. See docs/rules/RESPONSE_MEDIA_TYPE_ADDED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "RESPONSE_MEDIA_TYPE_ADDED",
    request: Severity::Review,
    response: Severity::Review,
    summary: "Response media type added",
    explanation: "A client may not decode the additional response representation; the impact depends on negotiation.",
    documentation: "docs/rules/RESPONSE_MEDIA_TYPE_ADDED.md",
};
