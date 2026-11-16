# PARAMETER_LOCATION_CHANGED

Parameter location changed.

Request severity: **breaking**. Response severity: **breaking**.

A generated SDK may send this value in the old location rather than the required new location.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PARAMETER_LOCATION_CHANGED/baseline.json), [candidate](../../tests/fixtures/rules/PARAMETER_LOCATION_CHANGED/candidate.json), [expected findings](../../tests/fixtures/rules/PARAMETER_LOCATION_CHANGED/expected.json).
