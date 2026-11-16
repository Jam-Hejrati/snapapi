# OPTIONAL_PARAMETER_ADDED

Optional parameter added.

Request severity: **compatible**. Response severity: **compatible**.

Existing client calls remain valid without the new optional argument.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/OPTIONAL_PARAMETER_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/OPTIONAL_PARAMETER_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/OPTIONAL_PARAMETER_ADDED/expected.json).
