# PROPERTY_ADDED

Property added.

Request severity: **compatible**. Response severity: **compatible**.

Existing clients are expected to tolerate this optional request property or ignore this additional response field.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PROPERTY_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/PROPERTY_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/PROPERTY_ADDED/expected.json).
