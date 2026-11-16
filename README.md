# SnapAPI

Detect potentially breaking changes between two local OpenAPI 3.0 or 3.1 specifications, with explanations for typed frontend clients and generated SDKs.

## Install and run

Rust 1.89 or newer is required.

```bash
cargo install --path .
snapapi compare baseline.yaml candidate.yaml
snapapi compare baseline.yaml candidate.yaml --format json --fail-on review
snapapi compare baseline.yaml candidate.yaml --format markdown --output report.md
snapapi version
```

Supported options: `--format text|json|markdown`, `--fail-on none|breaking|review|any-change`, `--config <path>`, `--max-bytes <size>`, `--output <path>`, `--quiet`, and `--verbose`. The defaults are text output, failure on breaking changes, and a 32 MiB size limit per input. Informational findings are included in JSON; text and Markdown display them with `--verbose`. Quiet mode suppresses stdout while keeping unsupported-feature diagnostics on stderr.

Every finding has a stable rule ID, classification, source location, before/after values, consumer-impact explanation, and a [rule documentation link](docs/rules.md). Shared-schema changes list all affected operations. The failure policy never changes severity.

## Supported comparison

Operations match by method and path shape: `/users/{id}` matches `/users/{userId}`. Literal path changes become an operation removal and addition. Parameter inheritance and header-name case are handled during normalization; path parameter names are matched by their position in the template.

Rules cover operations, anonymous authentication requirements, parameters, request/response media types, response status codes, schema types and simple nullability, properties and requiredness, enums, arrays, and common bounds. Property `readOnly`/`writeOnly` flags are considered in their request/response direction. See [all rules and their assumptions](docs/rules.md).

JSON reports use `schema_version: 1` and include summary counts, findings, unsupported-feature diagnostics, and `policy_violated`. An absent before/after value is omitted; explicit null is serialized as `null`. Counts refer to grouped changes rather than affected-operation counts. Examples/defaults are redacted, and large before/after snapshots are omitted.

## Configuration

Configuration is explicit, local YAML or JSON, limited to 1 MiB. Unknown fields, duplicate keys, invalid values, and unsupported versions are errors. CLI flags override valid configuration values.

```yaml
version: 1
policy:
  fail_on: breaking
output:
  format: text
comparison:
  include_informational: false
  max_bytes: 33554432
  unresolved_references: error
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Within the failure threshold |
| 1 | Failure policy violated |
| 2 | Invalid arguments or configuration |
| 3 | Input could not be read, parsed, or normalized |
| 4 | Internal or report-output error |

## Limits

This is a limited contract comparison tool, not a complete OpenAPI or JSON Schema validator. No remote/external references are fetched. Unsupported or unresolved references in supported contract areas fail with exit 3; structural reference cycles also fail. Recursive schema graphs are preserved and compared with cycle protection and a 128-level traversal limit.

Webhooks, callbacks, links, authentication scheme definitions, response headers, parameter serialization options, and media encoding are disclosed as unsupported. Complex `allOf`/`oneOf`/`anyOf` changes receive one review finding; only the simple nullable union form is analyzed. Boolean/null schemas, semantic schema-reference siblings, arbitrary pattern replacements, exclusive-bound changes, divisibility, and additional JSON Schema keywords receive conservative review findings. Security requirement scope/scheme-name changes need review; adding authentication to a previously anonymous operation is breaking. OpenAPI 3.1 can omit `paths`; 3.0 must provide it.

## Development

```bash
cargo test --all-targets --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo fmt --all -- --check
cargo rustdoc --lib --locked -- -D warnings
cargo +1.89.0 test --all-targets --locked
cargo audit
```

CI checks tests, formatting, Clippy, documentation, minimum Rust compatibility, and dependency advisories. Each rule has a module, doc, and baseline/candidate fixture under `tests/fixtures/rules/`; confirmed bugs have regression fixtures.
