# SECURITY_REQUIREMENT_CHANGED

Security requirement changed.

Request severity: **review**. Response severity: **review**.

A frontend client may need different credentials or scopes; the impact depends on authorization policy.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/SECURITY_REQUIREMENT_CHANGED/baseline.json), [candidate](../../tests/fixtures/rules/SECURITY_REQUIREMENT_CHANGED/candidate.json), [expected findings](../../tests/fixtures/rules/SECURITY_REQUIREMENT_CHANGED/expected.json).
