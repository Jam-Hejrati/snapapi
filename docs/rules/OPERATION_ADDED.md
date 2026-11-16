# OPERATION_ADDED

Operation added.

Request severity: **compatible**. Response severity: **compatible**.

Existing clients keep their operations; a generated SDK can expose this additional operation.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/OPERATION_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/OPERATION_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/OPERATION_ADDED/expected.json).
