# REQUEST_MEDIA_TYPE_ADDED

Request media type added.

Request severity: **compatible**. Response severity: **compatible**.

Existing payload formats remain accepted; clients have an additional content type available.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/REQUEST_MEDIA_TYPE_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/REQUEST_MEDIA_TYPE_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/REQUEST_MEDIA_TYPE_ADDED/expected.json).
