# OPERATION_DEPRECATED

Operation deprecated.

Request severity: **informational**. Response severity: **informational**.

The operation remains available; typed clients and SDK consumers should plan their migration.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/OPERATION_DEPRECATED/baseline.json), [candidate](../../tests/fixtures/rules/OPERATION_DEPRECATED/candidate.json), [expected findings](../../tests/fixtures/rules/OPERATION_DEPRECATED/expected.json).
