# SCHEMA_TYPE_CHANGED

Schema type changed.

Request severity: **breaking**. Response severity: **breaking**.

A generated SDK or TypeScript client may use the old type; request validation or response deserialization may fail.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/SCHEMA_TYPE_CHANGED/baseline.json), [candidate](../../tests/fixtures/rules/SCHEMA_TYPE_CHANGED/candidate.json), [expected findings](../../tests/fixtures/rules/SCHEMA_TYPE_CHANGED/expected.json).
