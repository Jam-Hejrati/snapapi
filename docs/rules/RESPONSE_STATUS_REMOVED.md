# RESPONSE_STATUS_REMOVED

Response status removed.

Request severity: **breaking**. Response severity: **breaking**.

A typed client may depend on handling this response status and its payload.

Classification is independent of the failure policy. SnapAPI reasons about declared contracts, not observed client code. Request property removal is `review` when the candidate permits unknown properties. Complex schema composition and changes with uncertain semantics are classified as `review`.

Regression fixture: [baseline](../../tests/fixtures/rules/RESPONSE_STATUS_REMOVED/baseline.json), [candidate](../../tests/fixtures/rules/RESPONSE_STATUS_REMOVED/candidate.json), [expected findings](../../tests/fixtures/rules/RESPONSE_STATUS_REMOVED/expected.json).
