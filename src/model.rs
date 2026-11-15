//! Deterministic normalized representation of the supported OpenAPI surface.
//!
//! Structural reference objects (path items, parameters, request bodies and
//! responses) are resolved locally and shallowly. Schema values are retained
//! as written, including `$ref` fields, so recursive schema graphs can be
//! compared later with visited-pair cycle protection instead of being eagerly
//! expanded during normalization.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::input::{Document, OpenApiVersion};

#[derive(Debug, Clone, PartialEq)]
pub enum Presence<T> {
    Absent,
    Null,
    Value(T),
}

impl<T> Presence<T> {
    fn from_object(object: &serde_json::Map<String, Value>, key: &str) -> Self
    where
        T: From<Value>,
    {
        match object.get(key) {
            None => Presence::Absent,
            Some(Value::Null) => Presence::Null,
            Some(value) => Presence::Value(T::from(value.clone())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HttpMethod {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

impl HttpMethod {
    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "get" => Self::Get,
            "put" => Self::Put,
            "post" => Self::Post,
            "delete" => Self::Delete,
            "options" => Self::Options,
            "head" => Self::Head,
            "patch" => Self::Patch,
            "trace" => Self::Trace,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::Put => "put",
            Self::Post => "post",
            Self::Delete => "delete",
            Self::Options => "options",
            Self::Head => "head",
            Self::Patch => "patch",
            Self::Trace => "trace",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OperationKey {
    pub method: HttpMethod,
    pub path_shape: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: Presence<Value>,
    pub location: Presence<Value>,
    pub required: Presence<Value>,
    pub schema: Presence<Schema>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestBody {
    pub required: Presence<Value>,
    pub content: Presence<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Response {
    pub description: Presence<Value>,
    pub content: Presence<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    /// Raw schema JSON. Local schema references are intentionally not expanded.
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Operation {
    pub key: OperationKey,
    pub method: HttpMethod,
    pub path: String,
    pub source: PathBuf,
    pub location: String,
    pub operation_id: Presence<Value>,
    pub parameters: Presence<BTreeMap<String, Parameter>>,
    pub request_body: Presence<RequestBody>,
    pub responses: Presence<BTreeMap<String, Response>>,
    pub raw: Value,
}

#[derive(Debug, Clone)]
pub struct ApiModel {
    pub version: OpenApiVersion,
    pub source: PathBuf,
    pub operations: BTreeMap<OperationKey, Operation>,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ModelError {
    #[error("{source_path}: {location}: expected an object at {field}")]
    InvalidShape {
        source_path: PathBuf,
        location: String,
        field: String,
    },
    #[error("{source_path}: {location}: expected {expected} at {field}")]
    InvalidField {
        source_path: PathBuf,
        location: String,
        field: String,
        expected: &'static str,
    },
    #[error("{source_path}: unsupported remote $ref `{reference}` at {location}")]
    RemoteRef {
        source_path: PathBuf,
        location: String,
        reference: String,
    },
    #[error("{source_path}: unsupported local $ref `{reference}` at {location}: {reason}")]
    UnsupportedLocalRef {
        source_path: PathBuf,
        location: String,
        reference: String,
        reason: &'static str,
    },
    #[error("{source_path}: unresolved local $ref `{reference}` at {location}")]
    UnresolvedRef {
        source_path: PathBuf,
        location: String,
        reference: String,
    },
    #[error("{source_path}: cyclic structural $ref `{reference}` at {location}")]
    RefCycle {
        source_path: PathBuf,
        location: String,
        reference: String,
    },
    #[error("{source_path}: ambiguous siblings beside Path Item $ref at {location}")]
    AmbiguousPathItemRef {
        source_path: PathBuf,
        location: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructuralRefKind {
    PathItem,
    ReferenceObject,
}

struct Resolver<'a> {
    root: &'a Value,
    source: &'a PathBuf,
    version: OpenApiVersion,
}

impl Resolver<'_> {
    fn resolve_path_item(&self, value: &Value, location: &str) -> Result<Value, ModelError> {
        self.resolve(value, location, "path item", StructuralRefKind::PathItem)
    }

    fn resolve_reference_object(
        &self,
        value: &Value,
        location: &str,
        field: &str,
    ) -> Result<Value, ModelError> {
        self.resolve(value, location, field, StructuralRefKind::ReferenceObject)
    }

    fn resolve(
        &self,
        value: &Value,
        location: &str,
        field: &str,
        kind: StructuralRefKind,
    ) -> Result<Value, ModelError> {
        let mut current = value.clone();
        let mut seen = BTreeSet::new();
        let mut overlays = Vec::new();

        loop {
            let object = current
                .as_object()
                .ok_or_else(|| ModelError::InvalidShape {
                    source_path: self.source.clone(),
                    location: location.into(),
                    field: field.into(),
                })?;
            let Some(raw_reference) = object.get("$ref") else {
                break;
            };
            let reference = raw_reference
                .as_str()
                .ok_or_else(|| ModelError::InvalidField {
                    source_path: self.source.clone(),
                    location: location.into(),
                    field: "$ref".into(),
                    expected: "a string",
                })?;

            if kind == StructuralRefKind::PathItem
                && object
                    .keys()
                    .any(|key| key != "$ref" && !key.starts_with("x-"))
            {
                return Err(ModelError::AmbiguousPathItemRef {
                    source_path: self.source.clone(),
                    location: location.into(),
                });
            }

            if kind == StructuralRefKind::ReferenceObject && self.version == OpenApiVersion::V3_1 {
                let overlay = ["summary", "description"]
                    .into_iter()
                    .filter_map(|key| object.get(key).map(|value| (key.into(), value.clone())))
                    .collect::<serde_json::Map<String, Value>>();
                overlays.push(overlay);
            }

            let target = self.reference_target(reference, location, &mut seen)?;
            current = target.clone();
        }

        if kind == StructuralRefKind::ReferenceObject
            && self.version == OpenApiVersion::V3_1
            && !overlays.is_empty()
        {
            let object = current
                .as_object_mut()
                .ok_or_else(|| ModelError::InvalidShape {
                    source_path: self.source.clone(),
                    location: location.into(),
                    field: field.into(),
                })?;
            for overlay in overlays.into_iter().rev() {
                object.extend(overlay);
            }
        }

        Ok(current)
    }

    fn reference_target<'a>(
        &'a self,
        reference: &str,
        location: &str,
        seen: &mut BTreeSet<String>,
    ) -> Result<&'a Value, ModelError> {
        if !reference.starts_with('#') {
            return Err(ModelError::RemoteRef {
                source_path: self.source.clone(),
                location: location.into(),
                reference: reference.into(),
            });
        }
        if reference.contains('%') {
            return Err(ModelError::UnsupportedLocalRef {
                source_path: self.source.clone(),
                location: location.into(),
                reference: reference.into(),
                reason: "percent-encoded fragments are not supported",
            });
        }
        let pointer = reference.strip_prefix('#').unwrap_or_default();
        if !pointer.is_empty() && !pointer.starts_with('/') {
            return Err(ModelError::UnsupportedLocalRef {
                source_path: self.source.clone(),
                location: location.into(),
                reference: reference.into(),
                reason: "expected a JSON Pointer fragment beginning with #/",
            });
        }
        if !seen.insert(reference.into()) {
            return Err(ModelError::RefCycle {
                source_path: self.source.clone(),
                location: location.into(),
                reference: reference.into(),
            });
        }
        self.root
            .pointer(pointer)
            .ok_or_else(|| ModelError::UnresolvedRef {
                source_path: self.source.clone(),
                location: location.into(),
                reference: reference.into(),
            })
    }
}

pub fn normalize(document: &Document) -> Result<ApiModel, ModelError> {
    let source = document.source.clone();
    let root = &document.root;
    let resolver = Resolver {
        root,
        source: &source,
        version: document.version,
    };
    let paths = root
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| ModelError::InvalidShape {
            source_path: source.clone(),
            location: String::new(),
            field: "paths".into(),
        })?;
    let mut operations = BTreeMap::new();

    for (path, raw_item) in paths {
        let path_location = format!("/paths/{}", escape(path));
        let item = resolver.resolve_path_item(raw_item, &path_location)?;
        let path_object = item.as_object().ok_or_else(|| ModelError::InvalidShape {
            source_path: source.clone(),
            location: path_location.clone(),
            field: "path item".into(),
        })?;
        let path_parameters =
            parse_parameters(path_object.get("parameters"), &resolver, &path_location)?;

        for (method_text, raw_operation) in path_object {
            let Some(method) = HttpMethod::parse(method_text) else {
                continue;
            };
            let location = format!("{path_location}/{method_text}");
            let object = raw_operation
                .as_object()
                .ok_or_else(|| ModelError::InvalidShape {
                    source_path: source.clone(),
                    location: location.clone(),
                    field: "operation".into(),
                })?;
            let operation_parameters =
                parse_parameters(object.get("parameters"), &resolver, &location)?;
            let parameters = merge_parameters(&path_parameters, operation_parameters);
            let request_body = parse_request_body(object.get("requestBody"), &resolver, &location)?;
            let responses = parse_responses(object.get("responses"), &resolver, &location)?;
            let key = OperationKey {
                method,
                path_shape: path_shape(path),
            };
            let operation = Operation {
                key: key.clone(),
                method,
                path: path.clone(),
                source: source.clone(),
                location,
                operation_id: Presence::from_object(object, "operationId"),
                parameters,
                request_body,
                responses,
                raw: raw_operation.clone(),
            };
            if operations.insert(key, operation).is_some() {
                return Err(ModelError::InvalidShape {
                    source_path: source.clone(),
                    location: path_location,
                    field: "duplicate operation path shape".into(),
                });
            }
        }
    }

    Ok(ApiModel {
        version: document.version,
        source,
        operations,
    })
}

fn parse_parameters(
    raw: Option<&Value>,
    resolver: &Resolver<'_>,
    at: &str,
) -> Result<Presence<BTreeMap<String, Parameter>>, ModelError> {
    let Some(raw) = raw else {
        return Ok(Presence::Absent);
    };
    if raw.is_null() {
        return Ok(Presence::Null);
    }
    let array = raw.as_array().ok_or_else(|| ModelError::InvalidShape {
        source_path: resolver.source.clone(),
        location: at.into(),
        field: "parameters".into(),
    })?;
    let mut result = BTreeMap::new();

    for (index, raw_parameter) in array.iter().enumerate() {
        let location = format!("{at}/parameters/{index}");
        let parameter = resolver.resolve_reference_object(raw_parameter, &location, "parameter")?;
        let object = parameter
            .as_object()
            .ok_or_else(|| ModelError::InvalidShape {
                source_path: resolver.source.clone(),
                location: location.clone(),
                field: "parameter".into(),
            })?;
        let name = required_string(object, "name", resolver.source, &location)?;
        let parameter_location = required_string(object, "in", resolver.source, &location)?;
        let key = parameter_key(&name, &parameter_location);
        result.insert(
            key,
            Parameter {
                name: Presence::Value(Value::String(name)),
                location: Presence::Value(Value::String(parameter_location)),
                required: Presence::from_object(object, "required"),
                schema: parse_schema_presence(object.get("schema")),
            },
        );
    }

    Ok(Presence::Value(result))
}

fn required_string(
    object: &serde_json::Map<String, Value>,
    field: &str,
    source: &Path,
    location: &str,
) -> Result<String, ModelError> {
    object
        .get(field)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| ModelError::InvalidField {
            source_path: source.to_path_buf(),
            location: location.into(),
            field: field.into(),
            expected: "a string",
        })
}

fn parameter_key(name: &str, location: &str) -> String {
    let normalized_name = if location.eq_ignore_ascii_case("header") {
        name.to_ascii_lowercase()
    } else {
        name.to_owned()
    };
    format!(
        "{}:{}",
        Value::String(location.to_owned()),
        Value::String(normalized_name)
    )
}

fn merge_parameters(
    inherited: &Presence<BTreeMap<String, Parameter>>,
    operation: Presence<BTreeMap<String, Parameter>>,
) -> Presence<BTreeMap<String, Parameter>> {
    match operation {
        Presence::Absent => inherited.clone(),
        Presence::Null => Presence::Null,
        Presence::Value(operation_parameters) => {
            let mut merged = match inherited {
                Presence::Value(parameters) => parameters.clone(),
                Presence::Absent | Presence::Null => BTreeMap::new(),
            };
            merged.extend(operation_parameters);
            Presence::Value(merged)
        }
    }
}

fn parse_schema_presence(raw: Option<&Value>) -> Presence<Schema> {
    match raw {
        None => Presence::Absent,
        Some(Value::Null) => Presence::Null,
        Some(value) => Presence::Value(Schema {
            value: value.clone(),
        }),
    }
}

fn parse_request_body(
    raw: Option<&Value>,
    resolver: &Resolver<'_>,
    at: &str,
) -> Result<Presence<RequestBody>, ModelError> {
    let Some(raw) = raw else {
        return Ok(Presence::Absent);
    };
    if raw.is_null() {
        return Ok(Presence::Null);
    }
    let location = format!("{at}/requestBody");
    let request_body = resolver.resolve_reference_object(raw, &location, "requestBody")?;
    let object = request_body
        .as_object()
        .ok_or_else(|| ModelError::InvalidShape {
            source_path: resolver.source.clone(),
            location: location.clone(),
            field: "requestBody".into(),
        })?;
    Ok(Presence::Value(RequestBody {
        required: Presence::from_object(object, "required"),
        content: Presence::from_object(object, "content"),
    }))
}

fn parse_responses(
    raw: Option<&Value>,
    resolver: &Resolver<'_>,
    at: &str,
) -> Result<Presence<BTreeMap<String, Response>>, ModelError> {
    let Some(raw) = raw else {
        return Ok(Presence::Absent);
    };
    if raw.is_null() {
        return Ok(Presence::Null);
    }
    let object = raw.as_object().ok_or_else(|| ModelError::InvalidShape {
        source_path: resolver.source.clone(),
        location: format!("{at}/responses"),
        field: "responses".into(),
    })?;
    let mut result = BTreeMap::new();

    for (code, raw_response) in object {
        let location = format!("{at}/responses/{}", escape(code));
        let response = resolver.resolve_reference_object(raw_response, &location, "response")?;
        let object = response
            .as_object()
            .ok_or_else(|| ModelError::InvalidShape {
                source_path: resolver.source.clone(),
                location: location.clone(),
                field: "response".into(),
            })?;
        result.insert(
            code.clone(),
            Response {
                description: Presence::from_object(object, "description"),
                content: Presence::from_object(object, "content"),
            },
        );
    }

    Ok(Presence::Value(result))
}

fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

pub fn path_shape(path: &str) -> String {
    let mut out = String::new();
    let mut in_param = false;
    for character in path.chars() {
        match character {
            '{' => {
                in_param = true;
                out.push_str("{}")
            }
            '}' => in_param = false,
            _ if !in_param => out.push(character),
            _ => {}
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shape_ignores_parameter_names() {
        assert_eq!(path_shape("/users/{id}"), path_shape("/users/{userId}"));
    }

    #[test]
    fn shape_preserves_literals() {
        assert_ne!(path_shape("/users/{id}"), path_shape("/accounts/{id}"));
    }

    #[test]
    fn pointer_escape_matches_json_pointer() {
        let root = serde_json::json!({"components":{"schemas":{"a/b~c":{"type":"string"}}}});
        let pointer = root.pointer("/components/schemas/a~1b~0c").unwrap();
        assert_eq!(pointer["type"], "string");
    }
}
