//! Failure policy: decides whether a set of findings should fail CI.
//!
//! The policy never changes a finding's severity. It only reads severities.

use crate::finding::Severity;

/// The `--fail-on` threshold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FailOn {
    /// Report only; never fail.
    None,
    /// Fail if any finding is breaking.
    #[default]
    Breaking,
    /// Fail if any finding is breaking or review.
    Review,
    /// Fail on any contract change (anything except informational).
    AnyChange,
}

impl FailOn {
    /// Returns true if the findings violate this policy.
    pub fn is_violated_by<I>(self, severities: I) -> bool
    where
        I: IntoIterator<Item = Severity>,
    {
        let threshold = match self {
            FailOn::None => return false,
            FailOn::Breaking => Severity::Breaking,
            FailOn::Review => Severity::Review,
            FailOn::AnyChange => Severity::Compatible,
        };
        severities.into_iter().any(|s| s >= threshold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Severity::*;

    #[test]
    fn none_never_fails() {
        assert!(!FailOn::None.is_violated_by([Breaking]));
    }

    #[test]
    fn breaking_ignores_review() {
        assert!(!FailOn::Breaking.is_violated_by([Review, Compatible]));
        assert!(FailOn::Breaking.is_violated_by([Compatible, Breaking]));
    }

    #[test]
    fn review_fails_on_review_or_breaking() {
        assert!(FailOn::Review.is_violated_by([Review]));
        assert!(!FailOn::Review.is_violated_by([Compatible, Informational]));
    }

    #[test]
    fn any_change_ignores_informational_only() {
        assert!(!FailOn::AnyChange.is_violated_by([Informational]));
        assert!(FailOn::AnyChange.is_violated_by([Compatible]));
    }

    #[test]
    fn empty_findings_never_fail() {
        assert!(!FailOn::AnyChange.is_violated_by([]));
    }
}
