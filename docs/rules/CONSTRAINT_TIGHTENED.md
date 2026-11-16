# CONSTRAINT_TIGHTENED

Constraint tightened.

Request severity: **breaking**. Response severity: **compatible**.

Requests previously accepted from a frontend may now fail validation. Response values stay within narrower limits.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/CONSTRAINT_TIGHTENED/baseline.json), [candidate](../../tests/fixtures/rules/CONSTRAINT_TIGHTENED/candidate.json), [expected findings](../../tests/fixtures/rules/CONSTRAINT_TIGHTENED/expected.json).
