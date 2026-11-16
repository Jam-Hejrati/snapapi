//! Property removed. See docs/rules/PROPERTY_REMOVED.md.

use super::Rule;
use crate::finding::Severity;

pub const RULE: Rule = Rule {
    id: "PROPERTY_REMOVED",
    request: Severity::Breaking,
    response: Severity::Breaking,
    summary: "Property removed",
    explanation: "A typed client may still write or read this property. Request removal is review unless the candidate rejects unknown properties.",
    documentation: "docs/rules/PROPERTY_REMOVED.md",
};

pub fn severity(direction: crate::finding::Direction, rejects_unknown: bool) -> Severity {
    if direction == crate::finding::Direction::Request && !rejects_unknown {
        Severity::Review
    } else {
        Severity::Breaking
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Direction;
    #[test]
    fn removing_a_request_property_depends_on_unknown_property_handling() {
        assert_eq!(severity(Direction::Request, false), Severity::Review);
        assert_eq!(severity(Direction::Request, true), Severity::Breaking);
        assert_eq!(severity(Direction::Response, false), Severity::Breaking);
    }
}
