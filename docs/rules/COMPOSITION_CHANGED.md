# COMPOSITION_CHANGED

Schema composition changed.

Request severity: **review**. Response severity: **review**.

A typed client may see a different union or intersection. Full composition semantics require human review.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/COMPOSITION_CHANGED/baseline.json), [candidate](../../tests/fixtures/rules/COMPOSITION_CHANGED/candidate.json), [expected findings](../../tests/fixtures/rules/COMPOSITION_CHANGED/expected.json).
