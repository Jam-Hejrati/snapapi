# PROPERTY_BECAME_OPTIONAL

Property became optional.

Request severity: **compatible**. Response severity: **breaking**.

Requests: clients may omit the field. Responses: clients typed against a required field may now encounter its absence.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PROPERTY_BECAME_OPTIONAL/baseline.json), [candidate](../../tests/fixtures/rules/PROPERTY_BECAME_OPTIONAL/candidate.json), [expected findings](../../tests/fixtures/rules/PROPERTY_BECAME_OPTIONAL/expected.json).
