# Prior art: OpenAPI breaking-change tools

_Last reviewed: 2026-11-09. Sources are linked at the bottom. Re-check them before the public beta, because these tools change often._

## Summary

The category already exists and is well served. **oasdiff** covers nearly all of the CLI scope in `docs/PLAN.md`, including:

- severity levels;
- `--fail-on`;
- per-check overrides;
- ignore files;
- OpenAPI 3.0 and 3.1;
- many output formats;
- a GitHub Action.

Rewriting a comparator in Rust is **not** a differentiator by itself. SnapAPI needs a narrower reason to exist (see "Chosen direction").

## Comparison

| | oasdiff | OpenAPITools/openapi-diff | pb33f openapi-changes |
|---|---|---|---|
| Language | Go (built on kin-openapi) | Java 8+ | Go (assumed; not stated on its About page) |
| License | Apache-2.0 | Apache-2.0 | not checked |
| OpenAPI versions | 3.0, 3.1 (docs also mention 3.2) | 3.0 officially; 3.1 is "not officially supported" (issue #755) | OpenAPI and AsyncAPI 3.x |
| Severity model | ERR / WARN / INFO | compatible / incompatible only | breaking flag; levels not documented |
| CI gating | `--fail-on ERR\|WARN\|INFO` | `--fail-on-incompatible`, `--fail-on-changed`; numeric exit codes not documented | not documented on the About page |
| Customization | per-check severity file (`err/warn/info/none`), ignore files, endpoint filters, config file, custom checks | YAML config toggles per check, Java SPI extensions, pluggable path matcher | not documented on the About page |
| Output | text, json, yaml, markdown, html, junit, githubactions, singleline; several languages | html, markdown, asciidoc, json, text | terminal, interactive TUI, json, markdown, html |
| Explanations | one-line change descriptions; a comment when a check's level is downgraded | a compatible/incompatible label per change | hierarchical change model; git history over time |
| Integrations | GitHub Action, pre-commit hook, hosted PR review, MCP server | Maven plugin, Docker | GitHub Action (linked) |
| Hosted / paid | oasdiff.com team PR review; `--open` uploads to oasdiff.com (opt-in); "Pro" tier | none | not stated |
| Stated gaps | no checks for callbacks or for parameter `content`; changes nested in allOf only WARN unless `--flatten-allof` | ambiguous path signatures make the path matcher fail; 3.1 support | n/a |

## Gaps we observed

1. **Consumer-impact explanations are thin.** oasdiff reports *what* changed, e.g. "removed the required property 'x' from the response". It does not explain what that change means for a typed client, such as a TypeScript type or a generated SDK. openapi-diff only labels each change compatible or incompatible.
2. **Rule rationale is spread out.** oasdiff documents its checks centrally, but the reasoning and caveats behind each one are not shown next to every finding.
3. **openapi-diff lags on 3.1** and has no severity levels in between.
4. **Optic** was reportedly archived in January 2026 (a secondhand claim we have not verified). Its users may be looking for a replacement.

## Chosen direction (confirmed 2026-11-09)

SnapAPI commits to four differentiators. The first two are part of the MVP core. The last two come after it.

1. **Frontend-first impact explanations (MVP).** Every finding states what it means for a typed consumer, e.g. "a TypeScript client generated from the baseline types `name` as `string`; it will now receive a number." The explanation is careful not to claim certainty about client code we never saw.
2. **Transparent, contributor-friendly rules (MVP).** Each rule has a short doc (ID, default severity, request/response direction, assumption, limitations), its own fixture, and a link to that doc from every finding. Adding a rule means adding one module, one doc, and one fixture.
3. **Schema-change grouping (after the core rules work).** When a shared component schema changes, SnapAPI reports it once and lists the operations it affects, instead of repeating the finding for every operation. The findings model has to record both the underlying change and the operations it affects from day one, even before the grouped output exists.
4. **Monorepo and git workflow (after the beta).** Find specs and compare them across git refs, e.g. `snapapi compare --base origin/main`, across several services, with little configuration.

Not differentiators on their own: speed, Rust, local-only processing. oasdiff is already local unless you pass `--open`.

## Sources
- [oasdiff README](https://github.com/oasdiff/oasdiff)
- [oasdiff BREAKING-CHANGES.md](https://github.com/oasdiff/oasdiff/blob/main/docs/BREAKING-CHANGES.md)
- [OpenAPITools/openapi-diff README](https://github.com/OpenAPITools/openapi-diff)
- [openapi-diff 3.1 discussion, issue #755 (mirror)](https://google.18789066745.workers.dev/OpenAPITools/openapi-diff/issues/755)
- [pb33f openapi-changes — About](https://pb33f.io/openapi-changes/about/)
- [Optic alternatives (DEV Community)](https://dev.to/peter_zsobrk_50d5c4ac7de/optic-is-dead-alternatives-for-openapi-breaking-change-detection-dmp)
