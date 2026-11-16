//! Deterministic normalized representation of the supported OpenAPI surface.
//!
//! Structural reference objects (path items, parameters, request bodies and
//! responses) are resolved locally and shallowly. Schema values are retained
//! as written, including `$ref` fields, so recursive schema graphs can be
//! compared later with visited-pair cycle protection instead of being eagerly
//! expanded during normalization.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
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
    pub content: Presence<Value>,
    pub source_location: String,
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
    pub raw: Value,
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
    pub security: Presence<Value>,
    pub raw: Value,
}

#[derive(Debug, Clone)]
pub struct ApiModel {
    pub version: OpenApiVersion,
    pub source: PathBuf,
    pub operations: BTreeMap<OperationKey, Operation>,
    /// Original document, including shared schemas and local reference targets.
    pub root: Value,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub source: PathBuf,
    pub location: String,
    pub feature: String,
    pub explanation: String,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ModelError {
    #[error("{source_path}: {location}: duplicate parameter name and location")]
    DuplicateParameter {
        source_path: PathBuf,
        location: String,
    },
    #[error("{source_path}: {location}: schema traversal exceeds the 128-level limit")]
    TraversalLimit {
        source_path: PathBuf,
        location: String,
    },
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
    diagnostics: RefCell<Vec<Diagnostic>>,
    validated_schemas: RefCell<BTreeSet<String>>,
}

impl Resolver<'_> {
    fn invalid_field(&self, location: &str, field: &str, expected: &'static str) -> ModelError {
        ModelError::InvalidField {
            source_path: self.source.clone(),
            location: location.into(),
            field: field.into(),
            expected,
        }
    }

    fn unsupported(&self, location: &str, feature: &str) {
        self.diagnostics.borrow_mut().push(Diagnostic {
            source: self.source.clone(),
            location: location.into(),
            feature: feature.into(),
            explanation:
                "This feature is retained but its compatibility semantics are not fully analyzed."
                    .into(),
        });
    }

    fn validate_field(
        &self,
        object: &serde_json::Map<String, Value>,
        field: &str,
        location: &str,
        expected: &'static str,
        accepts: fn(&Value) -> bool,
    ) -> Result<(), ModelError> {
        if let Some(value) = object.get(field)
            && !value.is_null()
            && !accepts(value)
        {
            return Err(self.invalid_field(
                &format!("{location}/{}", escape(field)),
                field,
                expected,
            ));
        }
        if object.get(field).is_some_and(Value::is_null) {
            self.unsupported(
                &format!("{location}/{}", escape(field)),
                &format!("explicit null {field}"),
            );
        }
        Ok(())
    }

    fn validate_content(
        &self,
        object: &serde_json::Map<String, Value>,
        location: &str,
    ) -> Result<(), ModelError> {
        self.validate_field(object, "content", location, "an object", Value::is_object)?;
        if let Some(content) = object.get("content").and_then(Value::as_object) {
            for (media_type, media) in content {
                if media_type.starts_with("x-") {
                    continue;
                }
                let at = format!("{location}/content/{}", escape(media_type));
                let media = media
                    .as_object()
                    .ok_or_else(|| self.invalid_field(&at, "media type", "an object"))?;
                if let Some(schema) = media.get("schema") {
                    self.validate_schema(schema, &format!("{at}/schema"), 0)?;
                }
                if media.contains_key("encoding") {
                    self.unsupported(&format!("{at}/encoding"), "encoding");
                }
            }
        }
        Ok(())
    }

    fn validate_schema(
        &self,
        value: &Value,
        location: &str,
        depth: usize,
    ) -> Result<(), ModelError> {
        if !self.validated_schemas.borrow_mut().insert(location.into()) {
            return Ok(());
        }
        if depth > 128 {
            return Err(ModelError::TraversalLimit {
                source_path: self.source.clone(),
                location: location.into(),
            });
        }
        if value.is_null() {
            self.unsupported(location, "explicit null schema");
            return Ok(());
        }
        if value.is_boolean() && self.version == OpenApiVersion::V3_1 {
            return Ok(());
        }
        let object = value.as_object().ok_or_else(|| {
            self.invalid_field(
                location,
                "schema",
                "a schema object (or a boolean in OpenAPI 3.1)",
            )
        })?;
        if let Some(reference) = object.get("$ref") {
            let reference = reference
                .as_str()
                .ok_or_else(|| self.invalid_field(location, "$ref", "a string"))?;
            let target = self.reference_target(reference, location, &mut BTreeSet::new())?;
            self.validate_schema(target, reference.trim_start_matches('#'), depth + 1)?;
            if self.version == OpenApiVersion::V3_0 {
                return Ok(());
            }
            if object.keys().any(|key| {
                key != "$ref"
                    && !matches!(
                        key.as_str(),
                        "description" | "summary" | "example" | "examples"
                    )
                    && !key.starts_with("x-")
            }) {
                self.unsupported(location, "schema $ref siblings");
            }
        }
        for field in [
            "nullable",
            "readOnly",
            "writeOnly",
            "deprecated",
            "uniqueItems",
        ] {
            self.validate_field(object, field, location, "a boolean", Value::is_boolean)?;
        }
        for field in ["description", "summary", "pattern", "format"] {
            self.validate_field(object, field, location, "a string", Value::is_string)?;
        }
        if let Some(types) = object.get("type").filter(|v| !v.is_null()) {
            let valid_type = |v: &Value| {
                matches!(
                    v.as_str(),
                    Some("object" | "array" | "string" | "number" | "integer" | "boolean")
                ) || (self.version == OpenApiVersion::V3_1 && v.as_str() == Some("null"))
            };
            let valid = valid_type(types)
                || (self.version == OpenApiVersion::V3_1
                    && types
                        .as_array()
                        .is_some_and(|a| !a.is_empty() && a.iter().all(valid_type)));
            if !valid {
                return Err(self.invalid_field(
                    location,
                    "type",
                    "a valid schema type or a 3.1 type array",
                ));
            }
        }
        for field in ["minimum", "maximum", "multipleOf"] {
            self.validate_field(object, field, location, "a number", Value::is_number)?;
        }
        for field in ["exclusiveMinimum", "exclusiveMaximum"] {
            let valid = if self.version == OpenApiVersion::V3_0 {
                Value::is_boolean
            } else {
                Value::is_number
            };
            self.validate_field(
                object,
                field,
                location,
                "a version-appropriate exclusive bound",
                valid,
            )?;
        }
        for field in [
            "minLength",
            "maxLength",
            "minItems",
            "maxItems",
            "minProperties",
            "maxProperties",
        ] {
            self.validate_field(object, field, location, "a nonnegative integer", |v| {
                v.as_u64().is_some()
            })?;
        }
        self.validate_field(object, "required", location, "an array of strings", |v| {
            v.as_array().is_some_and(|a| a.iter().all(Value::is_string))
        })?;
        self.validate_field(object, "enum", location, "an array", Value::is_array)?;
        if let Some(properties) = object.get("properties").filter(|v| !v.is_null()) {
            let properties = properties
                .as_object()
                .ok_or_else(|| self.invalid_field(location, "properties", "an object"))?;
            for (name, schema) in properties {
                self.validate_schema(
                    schema,
                    &format!("{location}/properties/{}", escape(name)),
                    depth + 1,
                )?;
            }
        }
        for field in ["items", "additionalProperties"] {
            if let Some(schema) = object.get(field) {
                // additionalProperties permits booleans in both versions.
                if field == "additionalProperties" && schema.is_boolean() {
                    continue;
                }
                self.validate_schema(schema, &format!("{location}/{field}"), depth + 1)?;
            }
        }
        for field in ["allOf", "oneOf", "anyOf"] {
            if let Some(schemas) = object.get(field).filter(|v| !v.is_null()) {
                let schemas = schemas
                    .as_array()
                    .ok_or_else(|| self.invalid_field(location, field, "an array of schemas"))?;
                if field == "allOf" || crate::schema::simple_nullable_variant(value).is_none() {
                    self.unsupported(&format!("{location}/{field}"), field);
                }
                for (i, schema) in schemas.iter().enumerate() {
                    self.validate_schema(schema, &format!("{location}/{field}/{i}"), depth + 1)?;
                }
            }
        }
        for field in object.keys() {
            if !crate::schema::supported_keyword(field) && !field.starts_with("x-") {
                self.unsupported(&format!("{location}/{}", escape(field)), field);
            }
        }
        for field in ["type", "properties"] {
            if object.get(field).is_some_and(Value::is_null) {
                self.unsupported(
                    &format!("{location}/{field}"),
                    &format!("explicit null {field}"),
                );
            }
        }
        Ok(())
    }

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
        diagnostics: RefCell::new(Vec::new()),
        validated_schemas: RefCell::new(BTreeSet::new()),
    };
    let empty_paths = serde_json::Map::new();
    let paths = match root.get("paths") {
        None if document.version == OpenApiVersion::V3_1 => &empty_paths,
        Some(Value::Object(paths)) => paths,
        _ => {
            return Err(ModelError::InvalidShape {
                source_path: source.clone(),
                location: "/paths".into(),
                field: "paths".into(),
            });
        }
    };
    if let Some(components) = root.get("components") {
        let components = components
            .as_object()
            .ok_or_else(|| resolver.invalid_field("/components", "components", "an object"))?;
        if let Some(schemas) = components.get("schemas") {
            let schemas = schemas.as_object().ok_or_else(|| {
                resolver.invalid_field("/components/schemas", "schemas", "an object")
            })?;
            for (name, schema) in schemas {
                resolver.validate_schema(
                    schema,
                    &format!("/components/schemas/{}", escape(name)),
                    0,
                )?;
            }
        }
        for field in ["callbacks", "links", "securitySchemes"] {
            if components.contains_key(field) {
                resolver.unsupported(&format!("/components/{field}"), field);
            }
        }
    }
    if root.get("webhooks").is_some() {
        resolver.unsupported("/webhooks", "webhooks");
    }
    let global_security = root.get("security");
    validate_security(global_security, &resolver, "/security")?;
    let mut operations = BTreeMap::new();

    for (path, raw_item) in paths {
        if path.starts_with("x-") {
            continue;
        }
        let path_location = format!("/paths/{}", escape(path));
        if !valid_path(path) {
            return Err(resolver.invalid_field(
                &path_location,
                "path",
                "a path beginning with / and balanced, nonempty template parameters",
            ));
        }
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
            resolver.validate_field(
                object,
                "operationId",
                &location,
                "a string",
                Value::is_string,
            )?;
            resolver.validate_field(
                object,
                "deprecated",
                &location,
                "a boolean",
                Value::is_boolean,
            )?;
            for field in ["description", "summary"] {
                resolver.validate_field(object, field, &location, "a string", Value::is_string)?;
            }
            if object.contains_key("callbacks") {
                resolver.unsupported(&format!("{location}/callbacks"), "callbacks");
            }
            validate_security(
                object.get("security"),
                &resolver,
                &format!("{location}/security"),
            )?;
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
                security: match object.get("security").or(global_security) {
                    None => Presence::Absent,
                    Some(Value::Null) => Presence::Null,
                    Some(value) => Presence::Value(value.clone()),
                },
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

    let mut diagnostics = resolver.diagnostics.into_inner();
    diagnostics.sort_by(|a, b| (&a.location, &a.feature).cmp(&(&b.location, &b.feature)));
    diagnostics.dedup();
    Ok(ApiModel {
        version: document.version,
        source,
        operations,
        root: root.clone(),
        diagnostics,
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
        resolver.unsupported(&format!("{at}/parameters"), "explicit null parameters");
        return Ok(Presence::Null);
    }
    let array = raw.as_array().ok_or_else(|| {
        resolver.invalid_field(&format!("{at}/parameters"), "parameters", "an array")
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
        if !matches!(
            parameter_location.as_str(),
            "path" | "query" | "header" | "cookie"
        ) {
            return Err(resolver.invalid_field(
                &format!("{location}/in"),
                "in",
                "path, query, header, or cookie",
            ));
        }
        resolver.validate_field(
            object,
            "required",
            &location,
            "a boolean",
            Value::is_boolean,
        )?;
        resolver.validate_content(object, &location)?;
        if let Some(schema) = object.get("schema") {
            resolver.validate_schema(schema, &format!("{location}/schema"), 0)?;
        }
        for field in ["style", "explode", "allowReserved", "allowEmptyValue"] {
            if object.contains_key(field) {
                resolver.unsupported(&format!("{location}/{field}"), field);
            }
        }
        let key = parameter_key(&name, &parameter_location);
        if result.contains_key(&key) {
            return Err(ModelError::DuplicateParameter {
                source_path: resolver.source.clone(),
                location,
            });
        }
        result.insert(
            key,
            Parameter {
                name: Presence::Value(Value::String(name)),
                location: Presence::Value(Value::String(parameter_location)),
                required: Presence::from_object(object, "required"),
                schema: parse_schema_presence(object.get("schema")),
                content: Presence::from_object(object, "content"),
                source_location: location,
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
        resolver.unsupported(&format!("{at}/requestBody"), "explicit null requestBody");
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
    resolver.validate_field(
        object,
        "required",
        &location,
        "a boolean",
        Value::is_boolean,
    )?;
    resolver.validate_content(object, &location)?;
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
        resolver.unsupported(&format!("{at}/responses"), "explicit null responses");
        return Ok(Presence::Null);
    }
    let object = raw.as_object().ok_or_else(|| ModelError::InvalidShape {
        source_path: resolver.source.clone(),
        location: format!("{at}/responses"),
        field: "responses".into(),
    })?;
    let mut result = BTreeMap::new();

    for (code, raw_response) in object {
        if code.starts_with("x-") {
            continue;
        }
        let location = format!("{at}/responses/{}", escape(code));
        if code != "default"
            && !(code.len() == 3
                && matches!(code.as_bytes()[0], b'1'..=b'5')
                && (code.as_bytes()[1..].iter().all(u8::is_ascii_digit) || &code[1..] == "XX"))
        {
            return Err(resolver.invalid_field(
                &location,
                "response status",
                "default, an HTTP status, or a status range such as 2XX",
            ));
        }
        let response = resolver.resolve_reference_object(raw_response, &location, "response")?;
        let object = response
            .as_object()
            .ok_or_else(|| ModelError::InvalidShape {
                source_path: resolver.source.clone(),
                location: location.clone(),
                field: "response".into(),
            })?;
        resolver.validate_field(
            object,
            "description",
            &location,
            "a string",
            Value::is_string,
        )?;
        resolver.validate_content(object, &location)?;
        for field in ["links", "headers"] {
            if object.contains_key(field) {
                resolver.unsupported(&format!("{location}/{field}"), field);
            }
        }
        result.insert(
            code.clone(),
            Response {
                description: Presence::from_object(object, "description"),
                content: Presence::from_object(object, "content"),
                raw: response.clone(),
            },
        );
    }

    Ok(Presence::Value(result))
}

pub(crate) fn escape(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

fn validate_security(
    raw: Option<&Value>,
    resolver: &Resolver<'_>,
    location: &str,
) -> Result<(), ModelError> {
    let Some(raw) = raw else {
        return Ok(());
    };
    if raw.is_null() {
        resolver.unsupported(location, "explicit null security");
        return Ok(());
    }
    let valid = raw.as_array().is_some_and(|requirements| {
        requirements.iter().all(|r| {
            r.as_object().is_some_and(|schemes| {
                schemes.values().all(|scopes| {
                    scopes
                        .as_array()
                        .is_some_and(|scopes| scopes.iter().all(Value::is_string))
                })
            })
        })
    });
    if !valid {
        return Err(resolver.invalid_field(
            location,
            "security",
            "an array of security objects with string scope arrays",
        ));
    }
    Ok(())
}

fn valid_path(path: &str) -> bool {
    if !path.starts_with('/') {
        return false;
    }
    let mut parameter_len = None;
    for character in path.chars() {
        match (character, parameter_len) {
            ('{', None) => parameter_len = Some(0),
            ('{', Some(_)) | ('}', None) | ('}', Some(0)) => return false,
            ('}', Some(_)) => parameter_len = None,
            (_, Some(n)) => parameter_len = Some(n + 1),
            _ => {}
        }
    }
    parameter_len.is_none()
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
