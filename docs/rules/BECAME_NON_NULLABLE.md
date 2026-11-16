# BECAME_NON_NULLABLE

Schema became non-nullable.

Request severity: **breaking**. Response severity: **compatible**.

A client sending null may now be rejected. Response clients receive a stronger non-null guarantee.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/BECAME_NON_NULLABLE/baseline.json), [candidate](../../tests/fixtures/rules/BECAME_NON_NULLABLE/candidate.json), [expected findings](../../tests/fixtures/rules/BECAME_NON_NULLABLE/expected.json).
