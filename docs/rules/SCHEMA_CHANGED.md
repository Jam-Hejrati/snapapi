# SCHEMA_CHANGED

Schema semantics changed.

Request severity: **review**. Response severity: **review**.

The change involves absent, null, boolean schemas, reference siblings, or a constraint whose impact cannot be safely inferred.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/SCHEMA_CHANGED/baseline.json), [candidate](../../tests/fixtures/rules/SCHEMA_CHANGED/candidate.json), [expected findings](../../tests/fixtures/rules/SCHEMA_CHANGED/expected.json).
