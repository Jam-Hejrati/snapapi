# SECURITY_REQUIREMENT_ADDED

Authentication required.

Request severity: **breaking**. Response severity: **breaking**.

Previously anonymous requests from a frontend client may now be rejected.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/SECURITY_REQUIREMENT_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/SECURITY_REQUIREMENT_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/SECURITY_REQUIREMENT_ADDED/expected.json).
