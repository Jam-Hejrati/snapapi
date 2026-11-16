# REQUIRED_PARAMETER_ADDED

Required parameter added.

Request severity: **breaking**. Response severity: **breaking**.

Existing client calls do not include the new required argument and may be rejected.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/REQUIRED_PARAMETER_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/REQUIRED_PARAMETER_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/REQUIRED_PARAMETER_ADDED/expected.json).
