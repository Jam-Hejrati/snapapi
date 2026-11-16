# REQUEST_MEDIA_TYPE_REMOVED

Request media type removed.

Request severity: **breaking**. Response severity: **breaking**.

A client sending this content type may now receive an unsupported-media-type error.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/REQUEST_MEDIA_TYPE_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/REQUEST_MEDIA_TYPE_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/REQUEST_MEDIA_TYPE_REMOVED/expected.json).
