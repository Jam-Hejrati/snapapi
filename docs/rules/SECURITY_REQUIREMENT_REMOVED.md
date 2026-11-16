# SECURITY_REQUIREMENT_REMOVED

Authentication requirement removed.

Request severity: **compatible**. Response severity: **compatible**.

Clients can still send authenticated requests; anonymous requests are now permitted.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/SECURITY_REQUIREMENT_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/SECURITY_REQUIREMENT_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/SECURITY_REQUIREMENT_REMOVED/expected.json).
