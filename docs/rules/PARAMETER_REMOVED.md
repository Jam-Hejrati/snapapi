# PARAMETER_REMOVED

Parameter removed.

Request severity: **breaking**. Response severity: **breaking**.

A generated SDK may still send this parameter; its value may be rejected or silently ignored.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PARAMETER_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/PARAMETER_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/PARAMETER_REMOVED/expected.json).
