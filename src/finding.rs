//! Core vocabulary shared by rules, reports and the exit policy.

/// How a change affects existing consumers.
///
/// Variants are ordered from least to most severe, so `Ord` comparisons can be
/// used by the exit policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Severity {
    /// Documentation-only change (descriptions, examples).
    Informational,
    /// Expected to keep existing clients working.
    Compatible,
    /// Impact depends on consumer behavior; a human should look.
    Review,
    /// A previously valid interaction may stop working.
    Breaking,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Informational => "informational",
            Severity::Compatible => "compatible",
            Severity::Review => "review",
            Severity::Breaking => "breaking",
        }
    }
}

/// Which side of the contract a schema belongs to.
///
/// Compatibility is direction-aware: narrowing what a request accepts breaks
/// clients, while narrowing what a response returns usually does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Request,
    Response,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_orders_from_informational_to_breaking() {
        assert!(Severity::Informational < Severity::Compatible);
        assert!(Severity::Compatible < Severity::Review);
        assert!(Severity::Review < Severity::Breaking);
    }
}
