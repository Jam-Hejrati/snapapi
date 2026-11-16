# OPERATION_REMOVED

Operation removed.

Request severity: **breaking**. Response severity: **breaking**.

A typed client or generated SDK calling this operation may no longer complete that interaction.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/OPERATION_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/OPERATION_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/OPERATION_REMOVED/expected.json).
