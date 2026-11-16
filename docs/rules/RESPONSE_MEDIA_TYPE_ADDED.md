# RESPONSE_MEDIA_TYPE_ADDED

Response media type added.

Request severity: **review**. Response severity: **review**.

A client may not decode the additional response representation; the impact depends on negotiation.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/RESPONSE_MEDIA_TYPE_ADDED/baseline.json), [candidate](../../tests/fixtures/rules/RESPONSE_MEDIA_TYPE_ADDED/candidate.json), [expected findings](../../tests/fixtures/rules/RESPONSE_MEDIA_TYPE_ADDED/expected.json).
