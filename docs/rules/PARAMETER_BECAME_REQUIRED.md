# PARAMETER_BECAME_REQUIRED

Parameter became required.

Request severity: **breaking**. Response severity: **breaking**.

A typed client that omits this argument may now receive a request validation error.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PARAMETER_BECAME_REQUIRED/baseline.json), [candidate](../../tests/fixtures/rules/PARAMETER_BECAME_REQUIRED/candidate.json), [expected findings](../../tests/fixtures/rules/PARAMETER_BECAME_REQUIRED/expected.json).
