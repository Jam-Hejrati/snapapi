# REQUEST_BODY_BECAME_OPTIONAL

Request body became optional.

Request severity: **compatible**. Response severity: **compatible**.

Existing frontend calls remain valid; callers may now omit the body.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/REQUEST_BODY_BECAME_OPTIONAL/baseline.json), [candidate](../../tests/fixtures/rules/REQUEST_BODY_BECAME_OPTIONAL/candidate.json), [expected findings](../../tests/fixtures/rules/REQUEST_BODY_BECAME_OPTIONAL/expected.json).
