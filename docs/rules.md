# SnapAPI v1 compatibility rules

This is the starting rule set, copied from the decisions in the approved implementation plan. Every rule has a stable ID. Once released, an ID must never be renamed or reused.

**Direction:**
- `req`: the rule applies to what the API accepts (parameters, request bodies).
- `resp`: the rule applies to what the API returns.
- `op`: the rule applies to whole operations.

**Severity:** `breaking`, `review`, `compatible`, or `informational`. Severity is separate from the `--fail-on` policy: the policy only decides whether CI fails, and never changes a finding's severity.

**Operation matching:** operations match by HTTP method plus path shape, so `/users/{id}` is the same operation as `/users/{userId}`. A path that changes shape (e.g. `/users/{id}` → `/accounts/{id}`) is reported as one `OPERATION_REMOVED` and one `OPERATION_ADDED`. SnapAPI does not try to detect renames.

## Operations

| ID | Dir | Default | Assumption |
|---|---|---|---|
| `OPERATION_REMOVED` | op | breaking | Existing callers can no longer call the operation. |
| `OPERATION_ADDED` | op | compatible | Existing operations are unaffected. |
| `OPERATION_DEPRECATED` | op | informational | The operation still works. Consumers should plan to migrate. |
| `SECURITY_REQUIREMENT_ADDED` | op | breaking | Anonymous callers that worked before may now be rejected. |

## Parameters (path, query, header, cookie)

| ID | Dir | Default | Assumption |
|---|---|---|---|
| `PARAMETER_REMOVED` | req | breaking | Clients that still send it may be rejected, or the value may be silently ignored. |
| `REQUIRED_PARAMETER_ADDED` | req | breaking | Existing requests don't include the new required value. |
| `OPTIONAL_PARAMETER_ADDED` | req | compatible | Existing requests stay valid. |
| `PARAMETER_BECAME_REQUIRED` | req | breaking | Requests that leave it out may be rejected. |
| `PARAMETER_BECAME_OPTIONAL` | req | compatible | The API now accepts more requests than before. |
| `PARAMETER_LOCATION_CHANGED` | req | breaking | The value is now expected somewhere else, e.g. moved from the query string to a header. |

## Request bodies

| ID | Dir | Default | Assumption |
|---|---|---|---|
| `REQUEST_BODY_BECAME_REQUIRED` | req | breaking | Requests without a body may be rejected. |
| `REQUEST_MEDIA_TYPE_REMOVED` | req | breaking | Clients sending that content type may be rejected. |
| `REQUEST_MEDIA_TYPE_ADDED` | req | compatible | The API accepts more content types than before. |

## Responses

| ID | Dir | Default | Assumption |
|---|---|---|---|
| `RESPONSE_STATUS_REMOVED` | resp | breaking | Clients may depend on handling that status code. *(This was "breaking or review" in the plan; it was set to breaking.)* |
| `RESPONSE_STATUS_ADDED` | resp | review | Clients may not handle the new status code. |
| `RESPONSE_MEDIA_TYPE_REMOVED` | resp | breaking | Clients that negotiate that content type lose it. |

## Schemas

Schema rules apply to the schemas of parameters, request bodies, and responses. For each rule, the `req` and `resp` columns give its default severity in requests and in responses.

| ID | req | resp | Assumption |
|---|---|---|---|
| `SCHEMA_TYPE_CHANGED` | breaking | breaking | Typed code and deserialization may fail. |
| `PROPERTY_REMOVED` | breaking¹ | breaking | Responses: consumers may read the property. |
| `PROPERTY_ADDED` | compatible² | compatible | Responses: clients ignore unknown fields. *(The plan contradicted itself here; it was set to compatible.)* |
| `PROPERTY_BECAME_REQUIRED` | breaking | compatible | Requests: payloads without the property may be rejected. Responses: the property is now guaranteed to be present. |
| `PROPERTY_BECAME_OPTIONAL` | compatible | breaking | Responses: clients may assume the property is always present. |
| `ENUM_VALUE_REMOVED` | breaking | compatible | Requests: a value that used to be accepted is now rejected. |
| `ENUM_VALUE_ADDED` | compatible | review | Responses: clients may handle enum values with an exhaustive switch. |
| `BECAME_NULLABLE` | compatible | breaking | Responses: clients may assume the value is never `null`. *(This was "breaking or review" in the plan; it was set to breaking.)* |
| `BECAME_NON_NULLABLE` | breaking | compatible | Requests: clients sending `null` may be rejected. |
| `CONSTRAINT_TIGHTENED` | breaking | compatible | Covers min/max, minLength/maxLength, pattern, and minItems/maxItems. |
| `CONSTRAINT_LOOSENED` | compatible | review | Responses: clients may rely on the old limits. |

¹ In a request, removing a property is breaking only when the schema rejects unknown properties (`additionalProperties: false`). Otherwise it is `review`.
² In a request, adding a property is compatible when the new property is optional. A newly added *required* property is reported as `PROPERTY_BECAME_REQUIRED`.

Nullability is read differently in each OpenAPI version:
- 3.0: `nullable: true`.
- 3.1: `type` includes `"null"`, or a `oneOf`/`anyOf` contains `{type: "null"}`. The `oneOf`/`anyOf` form is detected only in its simplest shape in v1.

## Informational (hidden from the default text output)

`DESCRIPTION_CHANGED`, `SUMMARY_CHANGED`, `EXAMPLE_CHANGED`.

## Not covered in v1 (reported as unsupported, never silently ignored)

- Remote or external `$ref`s.
- Callbacks.
- Webhooks (3.1).
- Links.
- Detailed semantics of `allOf`/`oneOf`/`anyOf` beyond simple nullability. A change inside these is reported as a single `review` finding.
- Swagger 2.0.
