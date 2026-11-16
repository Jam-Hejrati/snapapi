# REQUEST_BODY_BECAME_REQUIRED

Request body became required.

Request severity: **breaking**. Response severity: **breaking**.

Existing frontend calls without a body may now be rejected.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/REQUEST_BODY_BECAME_REQUIRED/baseline.json), [candidate](../../tests/fixtures/rules/REQUEST_BODY_BECAME_REQUIRED/candidate.json), [expected findings](../../tests/fixtures/rules/REQUEST_BODY_BECAME_REQUIRED/expected.json).
