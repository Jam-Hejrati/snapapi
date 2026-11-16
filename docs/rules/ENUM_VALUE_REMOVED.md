# ENUM_VALUE_REMOVED

Enum value removed.

Request severity: **breaking**. Response severity: **compatible**.

Requests: a value sent by an existing client may be rejected. Responses: the set of possible values becomes smaller.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/ENUM_VALUE_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/ENUM_VALUE_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/ENUM_VALUE_REMOVED/expected.json).
