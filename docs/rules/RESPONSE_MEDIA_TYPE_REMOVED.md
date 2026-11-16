# RESPONSE_MEDIA_TYPE_REMOVED

Response media type removed.

Request severity: **breaking**. Response severity: **breaking**.

A client negotiating this content type may lose the response representation it expects.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/RESPONSE_MEDIA_TYPE_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/RESPONSE_MEDIA_TYPE_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/RESPONSE_MEDIA_TYPE_REMOVED/expected.json).
