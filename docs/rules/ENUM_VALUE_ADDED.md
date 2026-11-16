# ENUM_VALUE_ADDED

Enum value added.

Request severity: **compatible**. Response severity: **review**.

Requests accept additional values. Response clients may use an exhaustive enum switch that lacks the new case.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/ENUM_VALUE_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/ENUM_VALUE_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/ENUM_VALUE_ADDED/expected.json).
