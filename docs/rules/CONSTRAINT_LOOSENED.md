# CONSTRAINT_LOOSENED

Constraint loosened.

Request severity: **compatible**. Response severity: **review**.

Requests accept a broader range. Response clients may rely on the previous limits and need to handle additional values.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/CONSTRAINT_LOOSENED/baseline.json), [candidate](../../tests/fixtures/rules/CONSTRAINT_LOOSENED/candidate.json), [expected findings](../../tests/fixtures/rules/CONSTRAINT_LOOSENED/expected.json).
