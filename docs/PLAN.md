# SnapAPI

## Product Specification, Logic, Features, and Implementation Roadmap

**Document status:** Proposal / pre-implementation
**Working name:** SnapAPI
**Product type:** Open-source API contract compatibility checker
**Primary implementation language:** Rust
**Initial interface:** Command-line interface (CLI)
**Primary audience:** Frontend and backend developers, API maintainers, platform teams, SDK maintainers, and CI owners

> **Important:** SnapAPI is a proposed project, not an existing released product. This document describes intended behavior and a practical implementation plan. Features marked for later releases are not part of the initial MVP.

---

## 1. Executive summary

SnapAPI compares two versions of an API contract and explains whether the proposed version may break existing consumers.

The initial focus is OpenAPI documents in JSON or YAML. A developer supplies:

- a **baseline** specification: the API contract consumers currently rely on;
- a **candidate** specification: the proposed contract after a change.

SnapAPI parses both documents, normalizes their structure, compares their API semantics rather than raw text, classifies each detected change, and produces a human-readable or machine-readable report.

The main promise is:

> **Catch potentially breaking API contract changes before they reach consumers or production.**

For example, changing a response field from `string` to `integer`, removing an endpoint, or adding a required query parameter can break existing clients even if the code still builds. SnapAPI should surface the change, explain why it matters, show the old and new values, and return an exit code that CI can use.

The long-term vision is a small, fast, trustworthy, contributor-friendly Rust tool that fits into local development, code review, and continuous integration.

## 2. The problem

API consumers and API providers often evolve separately.

A backend developer may:

- remove a field believed to be unused;
- rename a request property;
- make an optional parameter required;
- change a response type;
- add authentication to an operation;
- remove a response status code.

The backend may compile and its own tests may pass, while a frontend application, mobile client, third-party integration, or generated SDK fails.

A plain text diff is not enough. YAML key order, whitespace, descriptions, and formatting can change without changing API behavior. Conversely, one small schema edit can have a significant compatibility impact.

SnapAPI aims to answer four questions:

1. **What changed?**
2. **Where did it change?**
3. **How might the change affect existing consumers?**
4. **Should the local command or CI job fail?**

### 2.1 Who benefits?

- **Frontend developers:** detect when a backend change may invalidate UI assumptions or generated TypeScript clients.
- **Backend developers:** review the impact of contract changes before merging.
- **Full-stack teams:** make API changes visible across team boundaries.
- **SDK maintainers:** detect changes that can alter generated method signatures or response types.
- **Platform and DevEx teams:** enforce consistent compatibility rules in repositories and CI.
- **Open-source maintainers:** review API evolution across releases.

## 3. Product principles

SnapAPI should follow these principles:

1. **Semantic, not textual:** compare normalized API structures rather than raw lines.
2. **Explain every verdict:** each classification should link to a named rule and a concrete reason.
3. **Conservative but configurable:** distinguish definite compatibility risks from context-dependent warnings.
4. **Useful locally:** the core comparison must work without an account, hosted service, or database.
5. **CI-friendly:** deterministic output, stable exit codes, and machine-readable reports.
6. **Privacy-first:** local files should remain local by default; never upload specifications implicitly.
7. **Incremental delivery:** ship a narrow and dependable core before adding integrations.
8. **Contributor-friendly:** isolate comparison rules and format adapters so contributors can add tests and features without rewriting the engine.

## 4. Scope and non-goals

### 4.1 MVP scope

The first useful release should support:

- OpenAPI 3.0 and 3.1 input, subject to the chosen parser's actual support;
- JSON and YAML files;
- local files as inputs;
- endpoint and HTTP-method additions/removals;
- parameter additions/removals and basic schema changes;
- request body and response schema comparisons;
- a first set of compatibility rules;
- human-readable terminal output;
- JSON report output;
- configurable failure behavior for CI;
- useful diagnostics for invalid documents and unsupported features;
- automated unit and integration tests.

Support for Swagger 2.0, remote URL fetching, external references, HTML dashboards, and advanced plugin APIs should be added only after the core behavior is reliable.

### 4.2 Explicit non-goals for the first release

SnapAPI is not initially intended to:

- execute API requests against a live server;
- prove runtime behavior matches the specification;
- replace contract tests or integration tests;
- generate a complete SDK;
- replace OpenAPI validation and linting tools;
- guarantee that a change is safe for every possible consumer;
- manage API versions, publish documentation, or host API specifications;
- infer undocumented behavior that is absent from the contract;
- fetch arbitrary remote references automatically.

These boundaries prevent the first release from becoming an unmanageably large platform.

## 5. Core user experience

### 5.1 Basic command

Example proposed CLI syntax:

```bash
snapapi compare openapi-old.yaml openapi-new.yaml
```

The command should:

1. Read both files.
2. Detect JSON or YAML input from content and/or extension.
3. Parse the documents and report useful errors if parsing fails.
4. Validate the minimum structure required for comparison.
5. Normalize each document into an internal representation.
6. Match API operations between the baseline and candidate.
7. compare operation-level and schema-level structures.
8. Emit a list of changes, each with a location, old value, new value, severity, rule identifier, and explanation.
9. Summarize the result.
10. Return an exit code based on the configured policy.

Example illustrative output:

```text
SnapAPI — API compatibility report

Baseline:  openapi-old.yaml
Candidate: openapi-new.yaml

BREAKING  GET /users/{id}
  Response property removed: `email`
  Location: responses.200.application/json.schema.properties.email
  Rule: RESPONSE_PROPERTY_REMOVED
  Impact: Existing clients that read `email` may fail or lose expected data.

BREAKING  POST /users
  Request property is now required: `displayName`
  Location: requestBody.application/json.schema.required
  Rule: REQUEST_PROPERTY_BECAME_REQUIRED
  Impact: Existing clients that omit `displayName` may be rejected.

SAFE      GET /health
  New operation added.
  Rule: OPERATION_ADDED

Summary: 2 breaking, 0 review, 1 safe
Result: incompatible
```

The output above is illustrative. The exact wording and rule classifications should be defined by tests and documented before a stable release.

### 5.2 Compare directions

The baseline is the old contract and the candidate is the proposed new contract. Direction matters.

For a request, the API provider promises what it accepts. If the candidate accepts fewer request shapes, old clients may stop working.

For a response, the provider promises what clients can expect to receive. If the candidate may return values or shapes that old clients were not prepared to handle, compatibility may be affected.

The engine must therefore avoid applying the same simplistic rule to request and response schemas.

## 6. Classification model

Use a small, clearly documented set of categories at first.

### 6.1 Breaking

A change is **breaking** when the declared contract indicates that a previously valid consumer interaction may stop working without the consumer changing.

Potential examples:

- removing an operation;
- removing a request parameter;
- adding a required request parameter;
- making a request property required;
- removing a request enum value;
- narrowing accepted request values;
- changing a field to an incompatible type;
- removing a response property that consumers may read;
- adding a new required response property when old clients assume the previous response shape;
- changing a response schema so that it may contain an unexpected type or `null`;
- removing a response status code that clients rely on;
- adding an authentication requirement to a previously public operation.

Some of these cases depend on exact OpenAPI semantics and the comparison policy. Implement them only after the rule has a written rationale and tests.

### 6.2 Review

A change is **review** when its compatibility impact depends on consumer behavior, server behavior, or team conventions.

Potential examples:

- adding an enum value to a response when consumers may use exhaustive switches;
- adding a new response status code;
- changing a format annotation where runtimes enforce it differently;
- modifying a security scope in a way that depends on authorization policy;
- changing a default value where consumers or servers may interpret it differently.

Review is not a synonym for breaking. The report should explain the uncertainty rather than overstate certainty.

### 6.3 Compatible

A change is **compatible** when it is expected to preserve existing clients under the documented compatibility model.

Potential examples:

- adding a new independent operation;
- adding an optional request parameter;
- widening the values accepted by a request;
- adding a response property that old clients can ignore, assuming consumers tolerate additional properties;
- making a request property optional.

Even apparently safe changes can affect strict code generators, schema validators, or consumers with assumptions not expressed in the contract. The documentation must make the model's assumptions explicit.

### 6.4 Informational

An **informational** change does not ordinarily affect request/response compatibility.

Potential examples:

- changing descriptions or summaries;
- changing examples;
- changing documentation metadata;
- formatting changes that do not alter the parsed structure.

Information-level changes can be omitted from the default terminal report but remain available in JSON or verbose output.

### 6.5 Configurable policy

Do not make the rule engine silently change its definition of compatibility based on the release version. Use explicit configuration to control CI behavior, for example:

- fail only on breaking changes;
- fail on breaking or review changes;
- fail on any API contract change;
- report only, without failing.

The classification and the failure policy are separate concepts. A warning should not become a breaking change merely because a team chooses to fail CI on warnings.

## 7. Detailed comparison logic

The comparison engine should operate on normalized structures rather than serialized YAML or JSON text.

### Step 1: Load input

- Accept two local file paths.
- Read files with bounded, configurable size limits.
- Detect malformed or unsupported content.
- Preserve source paths in diagnostics.
- Avoid printing secrets that might appear in examples or server configuration.

### Step 2: Parse documents

Parse JSON or YAML into a structured representation.

Parsing errors should include:

- which input failed;
- line and column when the parser provides them;
- a short, actionable message;
- no stack trace by default.

The parser should not silently replace malformed values with defaults.

### Step 3: Normalize

Create a stable internal model for the parts of OpenAPI that SnapAPI supports.

Normalization should:

- normalize equivalent representations where safe;
- ignore irrelevant ordering differences;
- canonicalize HTTP methods;
- normalize path-template matching;
- distinguish absent, empty, and explicit `null` values where the format semantics require it;
- retain original locations so a report can point to the affected contract element;
- resolve supported local `$ref` references or report why they could not be resolved.

Do not normalize away meaningful distinctions. For example, an omitted field and an explicitly set field may have different semantics.

### Step 4: Match operations

Match operations primarily by HTTP method and path template.

For example, `GET /users/{id}` is compared with `GET /users/{userId}` as a candidate match because the path shape and method are the same. The parameter name itself still needs to be examined: a path parameter name is descriptive in the URL template, but query and header parameter names are part of the actual request.

A path shape change such as `/users/{id}` to `/accounts/{id}` should not automatically be treated as a rename. It may represent removal of one operation and addition of another, or a breaking path change. The initial implementation should choose and document a deterministic policy.

### Step 5: Compare operation-level elements

For each matched operation, compare:

- path and query parameters;
- headers and cookies where supported;
- required/optional status;
- parameter schemas;
- request body presence and requiredness;
- request media types;
- response status codes;
- response media types;
- response schemas;
- security requirements;
- deprecation state and relevant metadata.

The MVP can implement a subset, but unsupported areas must be disclosed in the report or documentation rather than silently ignored.

### Step 6: Compare schemas recursively

For each relevant schema, compare:

- types and type unions;
- nullability according to the OpenAPI version;
- required properties;
- property additions and removals;
- arrays and item schemas;
- enum values;
- numeric ranges;
- string length constraints and patterns;
- item-count constraints;
- `additionalProperties`;
- composition keywords such as `allOf`, `oneOf`, and `anyOf`, once semantics are tested;
- `$ref` targets.

Recursive schema comparison needs cycle protection. OpenAPI schemas can refer to themselves directly or indirectly. Track visited schema pairs to prevent infinite recursion and to avoid repeatedly reporting the same structural change.

### Step 7: Apply rules

Each detected structural difference should be passed to a rule that returns a structured finding.

A finding should contain:

- stable rule ID;
- severity/category;
- operation, if applicable;
- JSON Pointer or equivalent location;
- old value and new value, where safe and useful;
- short summary;
- explanation of possible consumer impact;
- optional remediation advice;
- optional confidence or caveat, if the project defines such a field.

Rules should be deterministic and independently testable.

### Step 8: Deduplicate and group

A single shared schema may be referenced by many operations. A schema change can have multiple consumer impacts.

The engine should distinguish:

- the underlying schema change;
- the operations affected by that change.

The report should avoid overwhelming users with redundant findings while retaining enough context to understand the impact. Whether findings are grouped by rule, schema, or operation should be configurable or consistently documented.

### Step 9: Render and decide exit status

Render findings into the requested output format. Then apply the configured failure threshold to the structured findings.

Do not make the exit status depend on human-readable text parsing.

## 8. Example compatibility rules

This table is a proposed starting policy, not a claim that every case is universally breaking.

| Change                              | Default classification                              | Reason                                                       |
| ----------------------------------- | --------------------------------------------------- | ------------------------------------------------------------ |
| Operation removed                   | Breaking                                            | Existing callers can no longer use the documented operation  |
| Operation added                     | Compatible                                          | Existing operations remain available                         |
| Required query parameter added      | Breaking                                            | Existing requests omit a newly required value                |
| Optional query parameter added      | Compatible                                          | Existing requests can continue without it                    |
| Request property becomes required   | Breaking                                            | Previously valid payloads may be rejected                    |
| Request enum value removed          | Breaking                                            | A formerly allowed value is no longer documented as accepted |
| Request constraints become stricter | Breaking                                            | Previously valid inputs may no longer be accepted            |
| Response property removed           | Breaking                                            | Consumers may rely on the property                           |
| Response property added             | Usually compatible                                  | Old consumers often ignore extra properties                  |
| Response enum gains a value         | Review                                              | Consumers may assume the set is exhaustive                   |
| Response type changes               | Breaking                                            | Consumer deserialization or typed code may fail              |
| Response becomes nullable           | Breaking or review, depending on exact schema       | Consumers may assume a non-null value                        |
| Response status code removed        | Breaking or review, depending on response semantics | Clients may rely on the status                               |
| Description or example changed      | Informational                                       | Documentation-only if semantics are unchanged                |
| Authentication requirement added    | Breaking                                            | Previously anonymous callers may be rejected                 |

A mature version should define direction-aware rules and edge cases precisely. Compatibility is not a universal property of a schema diff; it is evaluated against the project's stated consumer model.

## 9. CLI design

Proposed command structure:

```text
snapapi
  compare <baseline> <candidate>
  check <baseline> <candidate>
  validate <spec>
  rules list
  version
```

The first release may only need `compare` and `version`. Avoid implementing aliases and subcommands without a concrete user need.

### 9.1 `compare`

Purpose: compare two specifications and show findings.

Illustrative usage:

```bash
snapapi compare ./openapi.yaml ./openapi-next.yaml
snapapi compare main.yaml feature.yaml --format json
snapapi compare main.yaml feature.yaml --fail-on breaking
snapapi compare main.yaml feature.yaml --config .snapapi.yaml
```

Proposed flags:

- `--format text|json|markdown`
- `--config <path>`
- `--fail-on none|breaking|review|any-change`
- `--quiet`
- `--verbose`
- `--output <path>`

Exact names and defaults should be finalized before a stable CLI release.

### 9.2 `check`

A CI-oriented convenience command that uses a configured threshold and produces a concise summary. It may initially be an alias for `compare` with CI-friendly defaults.

### 9.3 `validate`

Validates a specification independently of comparison. This command should be added only if SnapAPI can offer validation beyond the parser's basic syntax checks. Avoid claiming full OpenAPI validation if the implementation only checks that the document parses.

### 9.4 Exit-code contract

A proposed stable contract:

| Exit code | Meaning                                                                  |
| --------: | ------------------------------------------------------------------------ |
|       `0` | Comparison completed and did not exceed the configured failure threshold |
|       `1` | Comparison completed but the configured policy was violated              |
|       `2` | Invalid arguments or configuration                                       |
|       `3` | Input could not be read, parsed, or normalized                           |
|       `4` | Internal error or unexpected failure                                     |

The precise mapping is a product decision. Once released, exit codes should be treated as a public interface and changed carefully.

## 10. Configuration file

A YAML configuration could look like this:

```yaml
version: 1

comparison:
  include_informational: false
  unresolved_references: error

policy:
  fail_on: breaking

rules:
  RESPONSE_ENUM_ADDED:
    severity: review
  DESCRIPTION_CHANGED:
    severity: informational

output:
  format: text
  color: auto
```

This is illustrative configuration. The implementation must define schema validation, supported keys, default values, and behavior for unknown fields.

Recommended rules:

- fail on unknown configuration keys by default, or emit a clear warning;
- document defaults;
- support command-line overrides;
- never make configuration precedence surprising;
- version the configuration format if breaking changes become necessary.

## 11. JSON report format

Machine-readable output is essential for CI and future integrations.

Illustrative JSON:

```json
{
  "tool": {
    "name": "snapapi",
    "version": "0.1.0"
  },
  "baseline": {
    "path": "openapi.yaml"
  },
  "candidate": {
    "path": "openapi-next.yaml"
  },
  "summary": {
    "breaking": 1,
    "review": 1,
    "compatible": 2,
    "informational": 0
  },
  "result": "incompatible",
  "changes": [
    {
      "rule_id": "REQUEST_PROPERTY_BECAME_REQUIRED",
      "severity": "breaking",
      "operation": "POST /users",
      "location": "requestBody.application/json.schema.required",
      "summary": "Request property became required",
      "before": false,
      "after": true,
      "explanation": "Existing clients may omit this property."
    }
  ]
}
```

The report schema should be versioned before external tools depend on it. Consider using JSON Schema to document the report format and test example reports against it.

Do not include credentials, authorization headers, or unnecessary sensitive example values in reports. Large or sensitive old/new values may need truncation or redaction options.

## 12. Architecture

Keep the application layered so the comparison engine can be tested without invoking the CLI.

```text
┌──────────────────────────────────────────┐
│ CLI / command parsing                    │
├──────────────────────────────────────────┤
│ Input loader and format detection        │
├──────────────────────────────────────────┤
│ Parser / OpenAPI adapter                 │
├──────────────────────────────────────────┤
│ Normalized internal representation       │
├──────────────────────────────────────────┤
│ Operation matcher                        │
├──────────────────────────────────────────┤
│ Structural comparison engine             │
├──────────────────────────────────────────┤
│ Compatibility rules and findings         │
├──────────────────────────────────────────┤
│ Report model → text / JSON / Markdown    │
├──────────────────────────────────────────┤
│ Exit-policy evaluator                    │
└──────────────────────────────────────────┘
```

### 12.1 Suggested Rust workspace

A workspace is useful once module boundaries become clear. Do not create many crates prematurely.

Possible later layout:

```text
snapapi/
├── Cargo.toml
├── crates/
│   ├── snapapi-core/
│   ├── snapapi-openapi/
│   ├── snapapi-report/
│   └── snapapi-cli/
├── tests/
│   ├── fixtures/
│   └── integration/
├── docs/
├── examples/
├── .github/
│   └── workflows/
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── LICENSE
└── README.md
```

Responsibilities:

- `snapapi-core`: normalized model, comparison abstractions, findings, and policy.
- `snapapi-openapi`: parsing adapter, reference handling, and OpenAPI normalization.
- `snapapi-report`: stable report model and output serializers.
- `snapapi-cli`: arguments, configuration, filesystem access, output, and exit codes.

For a very small MVP, begin as one crate with well-separated modules, then split it only when the boundaries are justified.

### 12.2 Candidate dependencies

Evaluate crate maintenance, license, supported OpenAPI versions, diagnostics, and API stability before selecting dependencies.

| Need                       | Candidate                                                     |
| -------------------------- | ------------------------------------------------------------- |
| CLI parsing                | `clap`                                                        |
| Serialization              | `serde`                                                       |
| JSON                       | `serde_json`                                                  |
| YAML                       | A maintained YAML crate compatible with the chosen data model |
| Errors in library code     | `thiserror`                                                   |
| Errors at the CLI boundary | `anyhow`, if helpful                                          |
| Logging/diagnostics        | `tracing` or a simpler approach if tracing is unnecessary     |
| Snapshot tests             | `insta`, optionally                                           |
| Temporary test files       | `tempfile`, if needed                                         |

Use an established OpenAPI parser if it meets the requirements. Writing a complete OpenAPI parser is not a good first milestone. The value of SnapAPI is its compatibility policy, explanation quality, and workflow—not parsing YAML from scratch.

## 13. Rust concepts this project teaches

| Rust concept                  | Where SnapAPI uses it                                                                    |
| ----------------------------- | ---------------------------------------------------------------------------------------- |
| Ownership and borrowing       | Passing parsed documents and schema nodes through the engine without unnecessary cloning |
| Structs and enums             | Representing operations, schemas, severity levels, and finding kinds                     |
| `Result` and error types      | Parsing, reference resolution, configuration, and I/O failures                           |
| Traits                        | Parser adapters, rule interfaces, and report renderers if abstraction is justified       |
| Iterators                     | Walking operations, parameters, properties, and findings                                 |
| Lifetimes                     | Borrowed views of the normalized model where they simplify ownership                     |
| `HashMap` / `BTreeMap`        | Matching operations and properties; choosing deterministic output order                  |
| Recursion and graph traversal | Nested schemas and `$ref` graphs                                                         |
| Testing                       | Rule unit tests, fixtures, regression cases, and CLI integration tests                   |
| Performance profiling         | Large specifications and deeply nested schemas                                           |
| Async Rust                    | Only if later features require concurrent network or service operations                  |

Do not add async solely to demonstrate advanced Rust. A local file comparison tool can be entirely synchronous and still teach a lot about Rust.

## 14. Testing strategy

Trustworthy compatibility decisions require a strong test suite.

### 14.1 Unit tests

Each rule should have focused tests:

- compatible case;
- breaking case;
- review case if applicable;
- absent values;
- explicit null values where relevant;
- malformed or unsupported schema;
- boundary conditions.

### 14.2 Fixture tests

Maintain pairs of baseline and candidate documents with expected findings. Include:

- simple API;
- nested objects;
- arrays;
- enums;
- required properties;
- reused `$ref` schemas;
- recursive schemas;
- multiple response codes;
- mixed JSON/YAML inputs;
- equivalent documents with different formatting;
- invalid documents;
- unsupported features.

### 14.3 Regression tests

Every confirmed bug should have a minimal fixture that fails before the fix and passes afterward.

### 14.4 Determinism tests

Run the same comparison repeatedly and verify the JSON and text reports are stable. Avoid output ordering that depends on hash-map iteration.

### 14.5 Fuzzing and robustness

As the project matures, consider fuzzing parsers and schema traversal, testing very deep schemas, and ensuring cyclic references cannot cause infinite recursion or stack exhaustion.

### 14.6 Performance benchmarks

Use representative public OpenAPI specifications and synthetic large inputs. Measure:

- total runtime;
- peak memory if tooling permits;
- normalization time;
- comparison time;
- report generation time.

Do not advertise performance superiority without reproducible benchmarks against comparable tools and clearly stated test conditions.

## 15. Security and privacy

API specifications may reveal internal paths, service names, server URLs, schemas, and example data.

Security defaults should include:

- local processing by default;
- no telemetry by default;
- no automatic upload of specifications;
- no automatic fetching of remote `$ref` values in the MVP;
- bounded input sizes and recursion depth;
- safe handling of malformed documents;
- redaction controls for potentially sensitive values;
- no evaluation of arbitrary code or expressions from a spec;
- careful escaping of terminal output to prevent control-sequence injection;
- dependency auditing and automated security checks.

If remote URL support is introduced later, design it defensively. Consider SSRF protections, redirect policies, request timeouts, response-size limits, and restrictions on private or link-local addresses. Do not send user credentials to arbitrary referenced URLs.

## 16. GitHub Actions integration

The first integration should be a simple workflow that checks the proposed specification against the version on the base branch.

Conceptual workflow:

```yaml
name: API compatibility

on:
  pull_request:

jobs:
  snapapi:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install SnapAPI
        run: |
          # Replace with the project's documented installation
          # command after release artifacts are available.
          echo "Install SnapAPI here"

      - name: Compare API contracts
        run: |
          git show origin/main:openapi.yaml > /tmp/openapi-base.yaml
          snapapi compare /tmp/openapi-base.yaml ./openapi.yaml \
            --fail-on breaking
```

This is a workflow sketch, not a ready-to-run installation guide. It assumes the specification is stored at `openapi.yaml` and that `origin/main` is the intended base. A production workflow must use the actual base branch/ref and handle missing or renamed files.

Later, SnapAPI could provide a maintained GitHub Action or post an annotated pull-request report. A native Action should be considered after the CLI behavior and output schema are stable.

## 17. Example end-to-end scenario

Suppose a team has this baseline response schema:

```yaml
type: object
required:
  - id
  - name
properties:
  id:
    type: string
  name:
    type: string
  email:
    type: string
```

The candidate changes it to:

```yaml
type: object
required:
  - id
  - name
properties:
  id:
    type: string
  name:
    type: integer
```

SnapAPI should report at least:

1. `name` changed from string to integer in a response schema;
2. `email` was removed from a response schema.

Both are likely breaking for consumers that rely on the original contract. The report should show the affected operation(s) and explain that a typed client may fail to compile or deserialize, or application code may lose expected data.

The implementation should not simply count changed YAML lines. It should identify schema properties and classify the semantic changes.

## 18. Roadmap

### Phase 0 — Validate the problem

Before coding extensively:

- inspect existing OpenAPI diff tools and their rule sets;
- read issue trackers and discussions to identify gaps;
- interview a few frontend/backend developers if possible;
- choose a specific differentiator;
- write down the first supported compatibility rules.

**Deliverable:** a one-page product scope and a set of acceptance tests.

### Phase 1 — Small, working core

- initialize the Rust project;
- choose and evaluate the parser;
- load two local documents;
- normalize paths and operations;
- detect added and removed operations;
- implement a small set of schema rules;
- produce readable output;
- add fixture-based tests.

**Exit criterion:** a developer can compare two real specs and get useful, explainable results.

### Phase 2 — Reliable CLI

- add CLI argument parsing;
- add JSON output;
- define stable exit codes;
- add configuration and explicit failure thresholds;
- improve errors and deterministic output;
- add CI for tests, formatting, and linting.

**Exit criterion:** a repository can run SnapAPI in CI and reliably fail on the configured policy.

### Phase 3 — Public beta

- write installation instructions;
- document supported OpenAPI features and known limitations;
- add example specifications and screenshots or terminal recordings;
- publish binaries for relevant platforms;
- add contribution guidelines and a code of conduct;
- create focused `good first issue` tasks;
- invite feedback and publish a changelog.

**Exit criterion:** users outside the maintainer's own machine can install, understand, and report issues against the tool.

### Phase 4 — Differentiation

Choose improvements based on user feedback, such as:

- clearer frontend-consumer impact explanations;
- baseline exceptions with expiration or justification;
- monorepo support;
- GitHub pull-request annotations;
- configurable rule packs;
- schema-change grouping;
- broader OpenAPI version support;
- improved `$ref` handling.

### Phase 5 — Optional ecosystem

Only if the core has real users:

- HTML report or React dashboard;
- SDK/type-generation integrations;
- plugin API;
- additional API contract formats;
- hosted history or team workflows.

Do not start with a hosted service. A useful local-first tool is easier to ship, test, and trust.

## 19. How to make the project distinct

OpenAPI comparison is already an established category. Existing tools include `oasdiff`, `openapi-diff`, and other spec-diff utilities. SnapAPI should not assume that rewriting a comparator in Rust is, by itself, a compelling differentiator.

Potential directions to validate:

1. **Frontend-first explanations:** explain what a change means for TypeScript clients, React applications, and generated SDKs without claiming certainty about code that was not inspected.
2. **Contributor-friendly rule engine:** make adding a compatibility rule straightforward, with rule documentation and test fixtures.
3. **Excellent local developer experience:** concise output, actionable messages, stable JSON, fast startup, and predictable exit codes.
4. **Monorepo and branch workflow:** compare the right base/candidate specs with minimal configuration.
5. **Transparent compatibility policy:** expose assumptions, rule IDs, and rationale so teams can trust and override decisions.
6. **Privacy-first local execution:** avoid uploading API contracts by default.

These are hypotheses to validate, not guaranteed market advantages. Before naming the product or investing in a large implementation, compare the feature sets and open issues of existing projects.

## 20. Contribution strategy

A good open-source project needs a maintainable way for contributors to make small, useful changes.

Suggested contribution areas:

- new rule + tests;
- parser compatibility improvements;
- additional report formats;
- better diagnostics;
- fixture coverage;
- documentation and examples;
- CI integrations;
- performance benchmarks;
- bug fixes for edge cases.

For every rule contribution, require:

1. a rule ID;
2. a short definition;
3. an explanation of the compatibility assumption;
4. a minimal before/after fixture;
5. an expected classification;
6. tests covering the rule;
7. a note about known limitations.

Use issue templates, pull-request templates, continuous integration, and a clear review policy. Label small self-contained tasks for first-time contributors. Respond respectfully and explain why a rule is or is not accepted.

## 21. Definition of a successful MVP

The MVP is successful when:

- a developer can install and run it from the terminal;
- it accepts two local OpenAPI documents in supported formats;
- it identifies a useful, documented subset of compatibility changes;
- every finding explains what changed and why it matters;
- output is deterministic;
- JSON output is stable and documented;
- exit codes work reliably in CI;
- invalid inputs fail with actionable diagnostics;
- tests cover all implemented rules;
- the README clearly states supported features and limitations;
- no network access is required for the core comparison;
- another developer can contribute a rule using the documentation.

Do not define success solely by stars or downloads. Early indicators include repeat usage, useful bug reports, external contributions, and teams adopting the CI check.

## 22. Risks and mitigations

| Risk                                           | Mitigation                                                              |
| ---------------------------------------------- | ----------------------------------------------------------------------- |
| Existing tools already solve the basic problem | Identify a real unmet need before implementing the full roadmap         |
| Compatibility is context-dependent             | Document assumptions and separate breaking changes from review findings |
| OpenAPI is complex                             | Reuse a maintained parser and ship a limited, explicit feature set      |
| Recursive references cause loops               | Track visited schema pairs and set safe traversal limits                |
| Reports become noisy                           | Group related findings and provide useful filters                       |
| CI results change unexpectedly                 | Add regression fixtures and version rule behavior carefully             |
| Project scope expands too quickly              | Keep a public roadmap and require evidence before adding major features |
| Contributors struggle to understand the code   | Document architecture, rule format, tests, and development setup        |
| Performance claims are unproven                | Publish reproducible benchmarks and avoid unsupported comparisons       |

## 23. Suggested first issues

These issues are small enough to be useful for an early public repository.

1. **Project skeleton:** create the Cargo package, CI workflow, formatting, and lint checks.
2. **Input parser:** load a JSON OpenAPI document and produce a structured error on invalid input.
3. **YAML input:** add YAML support and fixtures.
4. **Operation index:** index operations by method and path.
5. **Operation removal rule:** report a removed operation with a stable rule ID.
6. **Operation addition rule:** report an added operation.
7. **JSON report:** serialize findings into a documented schema.
8. **Exit-policy evaluator:** map findings to the configured CI threshold.
9. **Fixture harness:** compare baseline/candidate pairs against expected findings.
10. **Documentation:** explain supported features and how to add a rule.

## 24. Recommended first-week plan

**Day 1:** Read the OpenAPI overview and inspect several existing comparison tools. Record what SnapAPI will do differently.

**Day 2:** Create a Cargo project, CLI skeleton, README, license, and CI workflow.

**Day 3:** Select the OpenAPI parser and load two valid local documents. Handle invalid inputs well.

**Day 4:** Design the normalized operation model and match operations by method/path.

**Day 5:** Implement added/removed operation findings with unit tests and fixture tests.

**Day 6:** Add a first report renderer and deterministic ordering.

**Day 7:** Publish a short demo, document limitations, and ask developers for feedback before expanding the rule set.

The exact schedule depends on your Rust experience and the parser you choose. The goal of the first week is a small vertical slice, not complete OpenAPI compatibility analysis.

## 25. References and prior art

These links are useful for checking the current landscape and learning from established approaches:

- [OpenAPI.tools — Breaking Changes category](https://openapi.tools/categories/breaking-changes) — directory of tools for comparing API specifications.
- [oasdiff](https://www.oasdiff.com/diff) — an existing OpenAPI comparison tool with CLI and CI workflows.
- [oasdiff breaking-change documentation](https://www.oasdiff.com/docs/breaking-changes) — examples of how compatibility rules can be documented.
- [OpenAPITools/openapi-diff on GitHub](https://github.com/OpenAPITools/openapi-diff) — an established open-source implementation.
- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html) — the primary specification SnapAPI should follow.
- [GitHub: Contributing to open source](https://docs.github.com/en/get-started/exploring-projects-on-github/contributing-to-open-source) — guidance on making repositories easier to contribute to.

Existing projects demonstrate that the general problem is real, but also make differentiation essential. Review their latest feature sets, licenses, release activity, and issues before finalizing SnapAPI's scope.

---

## Final recommendation

Build **SnapAPI Core first**: local OpenAPI input, operation matching, a small set of direction-aware compatibility rules, deterministic text/JSON output, and CI exit codes.

Do not start with a dashboard, remote service, plugin system, or every possible OpenAPI rule. Make the initial comparison trustworthy, publish it, and let real developer feedback guide the next phase.

The strongest long-term product is not merely “an API diff tool written in Rust.” It is a tool developers can trust because it explains compatibility decisions, integrates cleanly into their workflow, and makes it easy for the community to improve its rules.
