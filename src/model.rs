//! Deterministic normalized representation of the supported OpenAPI surface.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

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
    #[error("{source_path}: unsupported remote $ref `{reference}` at {location}")]
    RemoteRef {
        source_path: PathBuf,
        location: String,
        reference: String,
    },
    #[error("{source_path}: unresolved local $ref `{reference}` at {location}")]
    UnresolvedRef {
        source_path: PathBuf,
        location: String,
        reference: String,
    },
    #[error("{source_path}: cyclic $ref `{reference}` at {location}")]
    RefCycle {
        source_path: PathBuf,
        location: String,
        reference: String,
    },
}

pub fn normalize(document: &Document) -> Result<ApiModel, ModelError> {
    let source = document.source.clone();
    let root = &document.root;
    let empty_location = String::new();
    let paths = root
        .get("paths")
        .and_then(Value::as_object)
        .ok_or_else(|| ModelError::InvalidShape {
            source_path: source.clone(),
            location: empty_location.clone(),
            field: "paths".into(),
        })?;
    let mut operations = BTreeMap::new();
    for (path, item) in paths {
        let path_obj = item.as_object().ok_or_else(|| ModelError::InvalidShape {
            source_path: source.clone(),
            location: format!("/paths/{}", escape(path)),
            field: "path item".into(),
        })?;
        for (method_text, raw) in path_obj {
            let Some(method) = HttpMethod::parse(method_text) else {
                continue;
            };
            let location = format!("/paths/{}/{}", escape(path), method_text);
            let resolved = resolve_tree(root, raw, &source, &location, &mut BTreeSet::new())?;
            let key = OperationKey {
                method,
                path_shape: path_shape(path),
            };
            let object = resolved
                .as_object()
                .expect("resolved operation retains object shape");
            let parameters = parse_parameters(object.get("parameters"), root, &source, &location)?;
            let request_body =
                parse_request_body(object.get("requestBody"), root, &source, &location)?;
            let responses = parse_responses(object.get("responses"), root, &source, &location)?;
            let op = Operation {
                key: key.clone(),
                method,
                path: path.clone(),
                source: source.clone(),
                location,
                operation_id: Presence::from_object(object, "operationId"),
                parameters,
                request_body,
                responses,
                raw: resolved,
            };
            if operations.insert(key.clone(), op).is_some() {
                return Err(ModelError::InvalidShape {
                    source_path: source.clone(),
                    location: format!("/paths/{}", escape(path)),
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
    root: &Value,
    source: &PathBuf,
    at: &str,
) -> Result<Presence<BTreeMap<String, Parameter>>, ModelError> {
    let Some(raw) = raw else {
        return Ok(Presence::Absent);
    };
    if raw.is_null() {
        return Ok(Presence::Null);
    }
    let Some(array) = raw.as_array() else {
        return Err(ModelError::InvalidShape {
            source_path: source.clone(),
            location: at.into(),
            field: "parameters".into(),
        });
    };
    let mut result = BTreeMap::new();
    for (i, p) in array.iter().enumerate() {
        let loc = format!("{at}/parameters/{i}");
        let p = resolve_tree(root, p, source, &loc, &mut BTreeSet::new())?;
        let o = p.as_object().ok_or_else(|| ModelError::InvalidShape {
            source_path: source.clone(),
            location: loc.clone(),
            field: "parameter".into(),
        })?;
        let name = Presence::from_object(o, "name");
        let locn = Presence::from_object(o, "in");
        let key = format!("{}:{}", value_key(&name), value_key(&locn));
        result.insert(
            key,
            Parameter {
                name,
                location: locn,
                required: Presence::from_object(o, "required"),
                schema: parse_schema_presence(o.get("schema"))?,
            },
        );
    }
    Ok(Presence::Value(result))
}
fn parse_schema_presence(raw: Option<&Value>) -> Result<Presence<Schema>, ModelError> {
    Ok(match raw {
        None => Presence::Absent,
        Some(Value::Null) => Presence::Null,
        Some(v) => Presence::Value(Schema { value: v.clone() }),
    })
}
fn parse_request_body(
    raw: Option<&Value>,
    root: &Value,
    source: &PathBuf,
    at: &str,
) -> Result<Presence<RequestBody>, ModelError> {
    let Some(raw) = raw else {
        return Ok(Presence::Absent);
    };
    if raw.is_null() {
        return Ok(Presence::Null);
    }
    let loc = format!("{at}/requestBody");
    let v = resolve_tree(root, raw, source, &loc, &mut BTreeSet::new())?;
    let o = v.as_object().ok_or_else(|| ModelError::InvalidShape {
        source_path: source.clone(),
        location: loc,
        field: "requestBody".into(),
    })?;
    Ok(Presence::Value(RequestBody {
        required: Presence::from_object(o, "required"),
        content: Presence::from_object(o, "content"),
    }))
}
fn parse_responses(
    raw: Option<&Value>,
    root: &Value,
    source: &PathBuf,
    at: &str,
) -> Result<Presence<BTreeMap<String, Response>>, ModelError> {
    let Some(raw) = raw else {
        return Ok(Presence::Absent);
    };
    if raw.is_null() {
        return Ok(Presence::Null);
    }
    let o = raw.as_object().ok_or_else(|| ModelError::InvalidShape {
        source_path: source.clone(),
        location: format!("{at}/responses"),
        field: "responses".into(),
    })?;
    let mut result = BTreeMap::new();
    for (code, r) in o {
        let loc = format!("{at}/responses/{}", escape(code));
        let v = resolve_tree(root, r, source, &loc, &mut BTreeSet::new())?;
        let o = v.as_object().ok_or_else(|| ModelError::InvalidShape {
            source_path: source.clone(),
            location: loc.clone(),
            field: "response".into(),
        })?;
        result.insert(
            code.clone(),
            Response {
                description: Presence::from_object(o, "description"),
                content: Presence::from_object(o, "content"),
            },
        );
    }
    Ok(Presence::Value(result))
}
fn value_key(v: &Presence<Value>) -> String {
    match v {
        Presence::Absent => "absent".into(),
        Presence::Null => "null".into(),
        Presence::Value(v) => v.to_string(),
    }
}

fn resolve_tree(
    root: &Value,
    value: &Value,
    source: &PathBuf,
    location: &str,
    seen: &mut BTreeSet<String>,
) -> Result<Value, ModelError> {
    if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
        if !reference.starts_with('#') {
            return Err(ModelError::RemoteRef {
                source_path: source.clone(),
                location: location.into(),
                reference: reference.into(),
            });
        }
        if !seen.insert(reference.into()) {
            return Err(ModelError::RefCycle {
                source_path: source.clone(),
                location: location.into(),
                reference: reference.into(),
            });
        }
        let pointer = reference.strip_prefix('#').unwrap_or("");
        let target = root
            .pointer(pointer)
            .ok_or_else(|| ModelError::UnresolvedRef {
                source_path: source.clone(),
                location: location.into(),
                reference: reference.into(),
            })?;
        let mut resolved = resolve_tree(root, target, source, location, seen)?;
        seen.remove(reference);
        if let (Some(base), Some(extra)) = (resolved.as_object_mut(), value.as_object()) {
            for (k, v) in extra {
                if k != "$ref" {
                    base.insert(k.clone(), v.clone());
                }
            }
        }
        return Ok(resolved);
    }
    match value {
        Value::Array(a) => Ok(Value::Array(
            a.iter()
                .enumerate()
                .map(|(i, v)| resolve_tree(root, v, source, &format!("{location}/{i}"), seen))
                .collect::<Result<_, _>>()?,
        )),
        Value::Object(o) => {
            let mut out = serde_json::Map::new();
            for (k, v) in o {
                out.insert(
                    k.clone(),
                    resolve_tree(root, v, source, &format!("{location}/{}", escape(k)), seen)?,
                );
            }
            Ok(Value::Object(out))
        }
        _ => Ok(value.clone()),
    }
}
fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}
pub fn path_shape(path: &str) -> String {
    let mut out = String::new();
    let mut in_param = false;
    for c in path.chars() {
        match c {
            '{' => {
                in_param = true;
                out.push_str("{}")
            }
            '}' => in_param = false,
            _ if !in_param => out.push(c),
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
        let p = root.pointer("/components/schemas/a~1b~0c").unwrap();
        assert_eq!(p["type"], "string");
    }
}
