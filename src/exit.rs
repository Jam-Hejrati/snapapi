//! The public exit-code contract (`docs/PLAN.md` §9.4).
//!
//! Treat these values as a stable interface: CI pipelines depend on them.

/// Process exit status returned by the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Exit {
    /// Comparison completed within the configured failure threshold.
    Ok = 0,
    /// Comparison completed but the failure policy was violated.
    PolicyViolated = 1,
    /// Invalid arguments or configuration.
    Usage = 2,
    /// An input could not be read, parsed or normalized.
    Input = 3,
    /// Internal error.
    Internal = 4,
}

impl From<Exit> for std::process::ExitCode {
    fn from(exit: Exit) -> Self {
        std::process::ExitCode::from(exit as u8)
    }
}
