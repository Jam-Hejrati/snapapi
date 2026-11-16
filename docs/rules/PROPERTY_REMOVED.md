# PROPERTY_REMOVED

Property removed.

Request severity: **breaking**. Response severity: **breaking**.

A typed client may still write or read this property. Request removal is review unless the candidate rejects unknown properties.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PROPERTY_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/PROPERTY_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/PROPERTY_REMOVED/expected.json).
