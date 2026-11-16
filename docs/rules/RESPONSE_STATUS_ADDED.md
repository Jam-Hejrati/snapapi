# RESPONSE_STATUS_ADDED

Response status added.

Request severity: **review**. Response severity: **review**.

Frontend status handling may be exhaustive and may not account for this new response.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/RESPONSE_STATUS_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/RESPONSE_STATUS_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/RESPONSE_STATUS_ADDED/expected.json).
