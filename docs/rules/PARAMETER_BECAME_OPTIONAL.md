# PARAMETER_BECAME_OPTIONAL

Parameter became optional.

Request severity: **compatible**. Response severity: **compatible**.

Client requests remain valid; a generated SDK can now allow the argument to be omitted.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/PARAMETER_BECAME_OPTIONAL/baseline.json), [candidate](../../tests/fixtures/rules/PARAMETER_BECAME_OPTIONAL/candidate.json), [expected findings](../../tests/fixtures/rules/PARAMETER_BECAME_OPTIONAL/expected.json).
