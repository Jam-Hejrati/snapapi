# PROPERTY_BECAME_REQUIRED

Property became required.

Request severity: **breaking**. Response severity: **compatible**.

Requests: existing typed clients may omit this field. Responses: clients now receive the field consistently.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PROPERTY_BECAME_REQUIRED/baseline.json), [candidate](../../tests/fixtures/rules/PROPERTY_BECAME_REQUIRED/candidate.json), [expected findings](../../tests/fixtures/rules/PROPERTY_BECAME_REQUIRED/expected.json).
