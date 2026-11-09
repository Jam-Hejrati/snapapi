# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project status

SnapAPI is a Rust CLI (pre-implementation) that compares two OpenAPI specs — a **baseline** (current contract) and a **candidate** (proposed contract) — and reports potentially breaking changes. The project is being built in phases, following the approved plan in docs/PLAN.md §18. So far only the base types exist:
- `finding::Severity` and `finding::Direction`;
- `policy::FailOn`;
- the `exit::Exit` codes.

There is no command-line argument parsing yet. The full product spec, rules, and roadmap live in [docs/PLAN.md](docs/PLAN.md); read the relevant section there before implementing a feature, since it defines intended behavior.

Other docs:
- [docs/rules.md](docs/rules.md) is the source of truth for v1 rule IDs and their default severities. Keep it in sync whenever a rule is added or changed.
- [docs/PRIOR_ART.md](docs/PRIOR_ART.md) explains why SnapAPI exists next to oasdiff. Its chosen differentiators should shape design decisions:
  1. frontend-first impact explanations in every finding;
  2. one doc and one fixture per rule, linked from findings;
  3. grouping findings by shared-schema change (the findings model must record both the underlying change and the operations it affects);
  4. git and monorepo workflows, after the beta.

The language is Rust (edition 2024, MSRV 1.85). The decision to stay with Rust rather than move to Go is final. Rust parsing is done with SnapAPI's own model over `serde_json::Value` instead of an OpenAPI crate, because no crate handles both 3.0 and 3.1.

## Commands

```bash
cargo build
cargo run -- compare old.yaml new.yaml   # planned CLI shape
cargo test
cargo test <test_name>                   # run a single test (substring match)
cargo clippy --all-targets
cargo fmt
```

## Intended architecture (from docs/PLAN.md)

The design is a layered pipeline. Keep the engine testable without going through the CLI:

CLI → input loader / format detection (JSON or YAML) → OpenAPI parser adapter → normalized internal model → operation matcher → recursive structural comparison → compatibility rules producing findings → report model (text / JSON / Markdown) → exit-policy evaluator.

Start with **one crate and well-separated modules**. Only split into a workspace (`snapapi-core`, `snapapi-openapi`, `snapapi-report`, `snapapi-cli`) once the boundaries are proven. Use an existing OpenAPI parser instead of writing one. The candidate crates are clap, serde, serde_json, thiserror (library code), anyhow (CLI boundary), and insta (snapshot tests). The tool should stay synchronous with no async.

Design invariants the plan requires:
- **Direction-aware rules.** Requests and responses follow opposite logic. Narrowing what a request accepts is breaking. Widening what a response may return is breaking or needs review.
- **Classification is separate from failure policy.** A finding's severity is one of `breaking`, `review`, `compatible`, or `informational`. `--fail-on none|breaking|review|any-change` only decides the exit code and never changes a finding's severity.
- **Each finding carries:** a stable rule ID (SCREAMING_SNAKE, e.g. `RESPONSE_PROPERTY_REMOVED`), severity, operation, location (JSON-Pointer-like), before/after values, summary, and explanation. Every rule must be deterministic and unit-testable on its own.
- **Operation matching** keys on HTTP method plus path shape, so `/users/{id}` and `/users/{userId}` match. A path change is never inferred as a rename.
- **Schema recursion** must protect against cycles by tracking visited schema pairs, because `$ref` graphs can be self-referential.
- **Deterministic output.** Use `BTreeMap` or sorted iteration so report order never depends on hash-map iteration.
- **Unsupported areas must be disclosed**, never silently ignored. Normalization must not erase the difference between absent, empty, and `null`.
- **The exit-code contract is public:** 0 = within threshold, 1 = policy violated, 2 = bad args/config, 3 = input unreadable/unparseable, 4 = internal error. The exit code must come from the structured findings, not from parsing rendered text.
- **Privacy.** Read only local files, with no implicit network access or remote `$ref` fetching in the MVP. Don't put secrets from examples in reports.

## Testing approach (planned)

- Each rule gets unit tests for its compatible, breaking, and review cases, plus absent and null values and boundary conditions.
- Fixture tests use baseline/candidate document pairs with expected findings, planned under `tests/fixtures/`.
- Every confirmed bug gets a minimal regression fixture.
- Determinism tests run the same comparison repeatedly and check that the output is identical.
