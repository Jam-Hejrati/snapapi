//! SnapAPI: detect potentially breaking changes between two OpenAPI specs.
//!
//! The pipeline (see `docs/PLAN.md` §12) is:
//! input → normalized model → operation matching → comparison → rules →
//! findings → report → exit policy.
//!
//! Modules are added phase by phase. The library contains everything except
//! argument parsing and process exit, so the engine can be tested without the
//! CLI.

pub mod exit;
pub mod finding;
pub mod policy;
