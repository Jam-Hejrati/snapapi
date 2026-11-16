# BECAME_NULLABLE

Schema became nullable.

Request severity: **compatible**. Response severity: **breaking**.

Requests may now contain null. A response client typed against a non-null value may now receive null.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/BECAME_NULLABLE/baseline.json), [candidate](../../tests/fixtures/rules/BECAME_NULLABLE/candidate.json), [expected findings](../../tests/fixtures/rules/BECAME_NULLABLE/expected.json).
