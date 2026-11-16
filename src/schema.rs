//! Recursive schema comparison with visited-pair protection and conservative fallbacks.

use crate::compare::{Finding, finding, metadata};
use crate::finding::Direction;
use crate::input::OpenApiVersion;
use crate::model::{ApiModel, Operation, escape};
use crate::rules::{self, Rule};
use serde_json::{Value, json};
use std::collections::BTreeSet;

#[allow(clippy::too_many_arguments)]
pub(crate) fn compare(
    baseline: &ApiModel,
    candidate: &ApiModel,
    old: &Value,
    new: &Value,
    old_location: &str,
    new_location: &str,
    operation: &Operation,
    direction: Direction,
) -> Vec<Finding> {
    let mut walker = Walker {
        baseline,
        candidate,
        operation,
        direction,
        visited: BTreeSet::new(),
        output: Vec::new(),
    };
    walker.visit(old, new, old_location, new_location, 0);
    walker.output
}

struct Walker<'a> {
    baseline: &'a ApiModel,
    candidate: &'a ApiModel,
    operation: &'a Operation,
    direction: Direction,
    visited: BTreeSet<(String, String)>,
    output: Vec<Finding>,
}

impl Walker<'_> {
    fn emit(&mut self, rule: Rule, location: &str, old: Option<Value>, new: Option<Value>) {
        self.output.push(finding(
            rule,
            Some(self.direction),
            self.operation,
            location,
            old,
            new,
        ));
    }

    fn visit(
        &mut self,
        old: &Value,
        new: &Value,
        old_location: &str,
        new_location: &str,
        depth: usize,
    ) {
        if !self
            .visited
            .insert((old_location.into(), new_location.into()))
        {
            if old != new && (old.get("$ref").is_some() != new.get("$ref").is_some()) {
                self.emit(
                    rules::SCHEMA_CHANGED,
                    new_location,
                    Some(old.clone()),
                    Some(new.clone()),
                );
            }
            return;
        }
        if depth > 128 {
            self.emit(rules::SCHEMA_CHANGED, new_location, None, None);
            return;
        }
        if old.is_null() || new.is_null() || old.is_boolean() || new.is_boolean() {
            if old != new {
                self.emit(
                    rules::SCHEMA_CHANGED,
                    new_location,
                    Some(old.clone()),
                    Some(new.clone()),
                );
            }
            return;
        }
        let old_ref = old.get("$ref").and_then(Value::as_str);
        let new_ref = new.get("$ref").and_then(Value::as_str);
        if old_ref.is_some() || new_ref.is_some() {
            let baseline = self.baseline;
            let candidate = self.candidate;
            let a = old_ref
                .and_then(|r| baseline.root.pointer(r.trim_start_matches('#')))
                .unwrap_or(old);
            let b = new_ref
                .and_then(|r| candidate.root.pointer(r.trim_start_matches('#')))
                .unwrap_or(new);
            let old_at = old_ref
                .map(|r| r.trim_start_matches('#'))
                .unwrap_or(old_location);
            let new_at = new_ref
                .map(|r| r.trim_start_matches('#'))
                .unwrap_or(new_location);
            let first_change = self.output.len();
            self.visit(a, b, old_at, new_at, depth + 1);
            let siblings = |value: &Value, version: OpenApiVersion| {
                let mut fields = value.as_object().cloned().unwrap_or_default();
                fields.remove("$ref");
                if version == OpenApiVersion::V3_0 {
                    fields.clear();
                }
                fields.retain(|key, _| {
                    !matches!(
                        key.as_str(),
                        "description" | "summary" | "example" | "examples"
                    ) && !key.starts_with("x-")
                });
                Value::Object(fields)
            };
            let old_siblings = if old_ref.is_some() {
                siblings(old, baseline.version)
            } else {
                json!({})
            };
            let new_siblings = if new_ref.is_some() {
                siblings(new, candidate.version)
            } else {
                json!({})
            };
            if old_siblings != json!({}) || new_siblings != json!({}) {
                for change in &mut self.output[first_change..] {
                    if change.severity != crate::finding::Severity::Informational {
                        change.rule_id = rules::SCHEMA_CHANGED.id;
                        change.severity = crate::finding::Severity::Review;
                        change.summary = rules::SCHEMA_CHANGED.summary.into();
                        change.explanation = "Schema reference siblings intersect with the target in OpenAPI 3.1; a typed client's compatibility needs human review.".into();
                        change.documentation = rules::SCHEMA_CHANGED.documentation;
                        change.change_id = format!(
                            "{}:{:?}:{}",
                            change.rule_id, change.direction, change.location
                        );
                    }
                }
                if self.output.len() == first_change && old != new {
                    self.emit(
                        rules::SCHEMA_CHANGED,
                        new_location,
                        Some(old.clone()),
                        Some(new.clone()),
                    );
                }
            }
            // $ref siblings intersect with the target in 3.1; shallow merging would erase constraints.
            if old_ref.is_some() && new_ref.is_some() {
                let a = old_siblings;
                let b = new_siblings;
                if a != b {
                    self.emit(rules::SCHEMA_CHANGED, new_location, Some(a), Some(b));
                }
                if candidate.version == OpenApiVersion::V3_1 {
                    metadata(old, new, self.operation, new_location, &mut self.output);
                }
            }
            return;
        }
        metadata(old, new, self.operation, new_location, &mut self.output);
        let nullable_old = nullable(old, self.baseline.version);
        let nullable_new = nullable(new, self.candidate.version);
        if nullable_old != nullable_new {
            self.emit(
                if nullable_new {
                    rules::BECAME_NULLABLE
                } else {
                    rules::BECAME_NON_NULLABLE
                },
                new_location,
                Some(json!(nullable_old)),
                Some(json!(nullable_new)),
            );
        }
        let old_base = simple_nullable_variant(old).unwrap_or(old);
        let new_base = simple_nullable_variant(new).unwrap_or(new);
        let old_type = types(old_base);
        let new_type = types(new_base);
        if old_type != new_type {
            self.emit(
                if old_type.as_ref().is_some_and(Value::is_null)
                    || new_type.as_ref().is_some_and(Value::is_null)
                {
                    rules::SCHEMA_CHANGED
                } else {
                    rules::SCHEMA_TYPE_CHANGED
                },
                &format!("{new_location}/type"),
                old_type,
                new_type,
            );
        }
        self.properties(old_base, new_base, old_location, new_location, depth);
        self.enums(old_base, new_base, new_location);
        self.constraints(old_base, new_base, new_location);
        if old_base.get("items").is_some() || new_base.get("items").is_some() {
            self.child(
                old_base.get("items"),
                new_base.get("items"),
                &format!("{old_location}/items"),
                &format!("{new_location}/items"),
                depth,
            );
        }
        let compositions = |v: &Value| {
            let mut fields = serde_json::Map::new();
            if simple_nullable_variant(v).is_none() {
                for key in ["allOf", "oneOf", "anyOf"] {
                    if let Some(value) = v.get(key) {
                        fields.insert(key.into(), value.clone());
                    }
                }
            }
            Value::Object(fields)
        };
        let a = compositions(old);
        let b = compositions(new);
        if a != b {
            self.emit(rules::COMPOSITION_CHANGED, new_location, Some(a), Some(b));
        }
        let keys: BTreeSet<_> = old
            .as_object()
            .into_iter()
            .flat_map(|o| o.keys())
            .chain(new.as_object().into_iter().flat_map(|o| o.keys()))
            .collect();
        for key in keys
            .into_iter()
            .filter(|key| !supported_keyword(key) && !key.starts_with("x-"))
        {
            if old.get(key) != new.get(key) {
                let snapshot = |v: Option<&Value>| {
                    if key == "default" {
                        v.map(|_| json!("<redacted>"))
                    } else {
                        v.cloned()
                    }
                };
                self.emit(
                    rules::SCHEMA_CHANGED,
                    &format!("{new_location}/{}", escape(key)),
                    snapshot(old.get(key)),
                    snapshot(new.get(key)),
                );
            }
        }
    }

    fn child(
        &mut self,
        old: Option<&Value>,
        new: Option<&Value>,
        old_location: &str,
        new_location: &str,
        depth: usize,
    ) {
        match (old, new) {
            (Some(a), Some(b)) => self.visit(a, b, old_location, new_location, depth + 1),
            (None, None) => {}
            (a, b) => self.emit(rules::SCHEMA_CHANGED, new_location, a.cloned(), b.cloned()),
        }
    }

    fn properties(
        &mut self,
        old: &Value,
        new: &Value,
        old_location: &str,
        new_location: &str,
        depth: usize,
    ) {
        let empty = serde_json::Map::new();
        let before_raw = old
            .get("properties")
            .and_then(Value::as_object)
            .unwrap_or(&empty);
        let after_raw = new
            .get("properties")
            .and_then(Value::as_object)
            .unwrap_or(&empty);
        let required = |value: &Value| {
            value
                .get("required")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect::<BTreeSet<_>>()
                })
                .unwrap_or_default()
        };
        let visible_old = |schema: &Value| visible_property(schema, self.baseline, self.direction);
        let visible_new = |schema: &Value| visible_property(schema, self.candidate, self.direction);
        let before: std::collections::BTreeMap<_, _> = before_raw
            .iter()
            .filter(|(_, schema)| visible_old(schema))
            .map(|(name, schema)| (name.clone(), schema))
            .collect();
        let after: std::collections::BTreeMap<_, _> = after_raw
            .iter()
            .filter(|(_, schema)| visible_new(schema))
            .map(|(name, schema)| (name.clone(), schema))
            .collect();
        let old_required: BTreeSet<_> = required(old)
            .into_iter()
            .filter(|name| before_raw.get(name).is_none_or(visible_old))
            .collect();
        let new_required: BTreeSet<_> = required(new)
            .into_iter()
            .filter(|name| after_raw.get(name).is_none_or(visible_new))
            .collect();
        for (name, a) in &before {
            let loc = format!("{new_location}/properties/{}", escape(name));
            if let Some(b) = after.get(name) {
                self.visit(
                    a,
                    b,
                    &format!("{old_location}/properties/{}", escape(name)),
                    &loc,
                    depth + 1,
                );
            } else {
                let mut item = finding(
                    rules::PROPERTY_REMOVED,
                    Some(self.direction),
                    self.operation,
                    &loc,
                    Some((*a).clone()),
                    None,
                );
                item.severity = rules::property_removed::severity(
                    self.direction,
                    new.get("additionalProperties") == Some(&Value::Bool(false)),
                );
                self.output.push(item);
            }
        }
        for (name, b) in &after {
            if !before.contains_key(name)
                && !(self.direction == Direction::Request && new_required.contains(name.as_str()))
            {
                self.emit(
                    rules::PROPERTY_ADDED,
                    &format!("{new_location}/properties/{}", escape(name)),
                    None,
                    Some((*b).clone()),
                );
            }
        }
        for name in old_required.union(&new_required) {
            let a = old_required.contains(name);
            let b = new_required.contains(name);
            if a != b && (b || after.contains_key(name) || !before.contains_key(name)) {
                self.emit(
                    if b {
                        rules::PROPERTY_BECAME_REQUIRED
                    } else {
                        rules::PROPERTY_BECAME_OPTIONAL
                    },
                    &format!("{new_location}/required/{}", escape(name)),
                    Some(json!(a)),
                    Some(json!(b)),
                );
            }
        }
        for key in ["properties", "required"] {
            if (old.get(key).is_some_and(Value::is_null)
                || new.get(key).is_some_and(Value::is_null))
                && old.get(key) != new.get(key)
            {
                self.emit(
                    rules::SCHEMA_CHANGED,
                    &format!("{new_location}/{key}"),
                    old.get(key).cloned(),
                    new.get(key).cloned(),
                );
            }
        }
    }

    fn enums(&mut self, old: &Value, new: &Value, location: &str) {
        match (old.get("enum"), new.get("enum")) {
            (Some(Value::Array(a)), Some(Value::Array(b))) => {
                let removed: Vec<_> = a.iter().filter(|v| !b.contains(v)).cloned().collect();
                let added: Vec<_> = b.iter().filter(|v| !a.contains(v)).cloned().collect();
                if !removed.is_empty() {
                    self.emit(
                        rules::ENUM_VALUE_REMOVED,
                        &format!("{location}/enum"),
                        Some(json!(removed)),
                        None,
                    );
                }
                if !added.is_empty() {
                    self.emit(
                        rules::ENUM_VALUE_ADDED,
                        &format!("{location}/enum"),
                        None,
                        Some(json!(added)),
                    );
                }
            }
            (None, Some(Value::Array(b))) => self.emit(
                rules::CONSTRAINT_TIGHTENED,
                &format!("{location}/enum"),
                None,
                Some(json!(b)),
            ),
            (Some(Value::Array(a)), None) => self.emit(
                rules::CONSTRAINT_LOOSENED,
                &format!("{location}/enum"),
                Some(json!(a)),
                None,
            ),
            (a, b) if a != b => self.emit(
                rules::SCHEMA_CHANGED,
                &format!("{location}/enum"),
                a.cloned(),
                b.cloned(),
            ),
            _ => {}
        }
    }

    fn constraints(&mut self, old: &Value, new: &Value, location: &str) {
        for (field, is_min) in [
            ("minimum", true),
            ("maximum", false),
            ("minLength", true),
            ("maxLength", false),
            ("minItems", true),
            ("maxItems", false),
            ("minProperties", true),
            ("maxProperties", false),
        ] {
            let zero_default = matches!(field, "minLength" | "minItems" | "minProperties");
            let zero = json!(0);
            let default = if zero_default { Some(&zero) } else { None };
            let a = old.get(field).filter(|v| v.is_number()).or(default);
            let b = new.get(field).filter(|v| v.is_number()).or(default);
            let order = match (a, b) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, Some(_)) => std::cmp::Ordering::Less,
                (Some(_), None) => std::cmp::Ordering::Greater,
                (Some(a), Some(b)) => number_cmp(a, b),
            };
            if old.get(field).is_some_and(Value::is_null)
                || new.get(field).is_some_and(Value::is_null)
            {
                if old.get(field) != new.get(field) {
                    self.emit(
                        rules::SCHEMA_CHANGED,
                        &format!("{location}/{field}"),
                        old.get(field).cloned(),
                        new.get(field).cloned(),
                    );
                }
            } else if order != std::cmp::Ordering::Equal {
                let tighter = match (a, b) {
                    (None, Some(_)) => true,
                    (Some(_), None) => false,
                    (Some(a), Some(b)) => {
                        if is_min {
                            number_cmp(b, a).is_gt()
                        } else {
                            number_cmp(b, a).is_lt()
                        }
                    }
                    _ => false,
                };
                self.emit(
                    if tighter {
                        rules::CONSTRAINT_TIGHTENED
                    } else {
                        rules::CONSTRAINT_LOOSENED
                    },
                    &format!("{location}/{field}"),
                    old.get(field).cloned(),
                    new.get(field).cloned(),
                );
            }
        }
        for field in ["exclusiveMinimum", "exclusiveMaximum", "multipleOf"] {
            if old.get(field) != new.get(field) {
                // Version-specific exclusive bounds and divisibility need a combined constraint analysis.
                self.emit(
                    rules::SCHEMA_CHANGED,
                    &format!("{location}/{field}"),
                    old.get(field).cloned(),
                    new.get(field).cloned(),
                );
            }
        }
        if old.get("pattern") != new.get("pattern") {
            let a = old.get("pattern");
            let b = new.get("pattern");
            let rule = match (a, b) {
                (None, Some(Value::String(_))) => rules::CONSTRAINT_TIGHTENED,
                (Some(Value::String(_)), None) => rules::CONSTRAINT_LOOSENED,
                _ => rules::SCHEMA_CHANGED,
            };
            self.emit(rule, &format!("{location}/pattern"), a.cloned(), b.cloned());
        }
        for field in ["uniqueItems", "additionalProperties"] {
            let a = old.get(field);
            let b = new.get(field);
            let permissive = |v: Option<&Value>| {
                if field == "uniqueItems" {
                    v != Some(&Value::Bool(true))
                } else {
                    v != Some(&Value::Bool(false))
                }
            };
            if a.is_some_and(Value::is_null) || b.is_some_and(Value::is_null) {
                if a != b {
                    self.emit(
                        rules::SCHEMA_CHANGED,
                        &format!("{location}/{field}"),
                        a.cloned(),
                        b.cloned(),
                    );
                }
            } else if permissive(a) != permissive(b) {
                self.emit(
                    if permissive(b) {
                        rules::CONSTRAINT_LOOSENED
                    } else {
                        rules::CONSTRAINT_TIGHTENED
                    },
                    &format!("{location}/{field}"),
                    a.cloned(),
                    b.cloned(),
                );
            } else if field == "additionalProperties"
                && (a.is_some_and(Value::is_object) || b.is_some_and(Value::is_object))
            {
                self.child(
                    a,
                    b,
                    &format!("{location}/{field}"),
                    &format!("{location}/{field}"),
                    0,
                );
            }
        }
    }
}

fn number_cmp(a: &Value, b: &Value) -> std::cmp::Ordering {
    if let (Some(a), Some(b)) = (a.as_u64(), b.as_u64()) {
        return a.cmp(&b);
    }
    if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
        return a.cmp(&b);
    }
    if a.as_i64().is_some_and(|n| n < 0) && b.as_u64().is_some() {
        return std::cmp::Ordering::Less;
    }
    if b.as_i64().is_some_and(|n| n < 0) && a.as_u64().is_some() {
        return std::cmp::Ordering::Greater;
    }
    a.as_f64()
        .partial_cmp(&b.as_f64())
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn types(value: &Value) -> Option<Value> {
    match value.get("type") {
        Some(Value::String(v)) if v != "null" => Some(json!(v)),
        Some(Value::Array(values)) => {
            let mut values: Vec<_> = values
                .iter()
                .filter_map(Value::as_str)
                .filter(|v| *v != "null")
                .collect();
            values.sort();
            values.dedup();
            match values.as_slice() {
                [] => None,
                [single] => Some(json!(single)),
                _ => Some(json!(values)),
            }
        }
        v => v.cloned(),
    }
}

fn nullable(value: &Value, version: OpenApiVersion) -> bool {
    match version {
        OpenApiVersion::V3_0 => {
            value.get("type").is_some() && value.get("nullable") == Some(&Value::Bool(true))
        }
        OpenApiVersion::V3_1 => {
            value.get("type") == Some(&json!("null"))
                || value
                    .get("type")
                    .and_then(Value::as_array)
                    .is_some_and(|a| a.contains(&json!("null")))
                || simple_nullable_variant(value).is_some()
        }
    }
}

pub(crate) fn simple_nullable_variant(value: &Value) -> Option<&Value> {
    for field in ["oneOf", "anyOf"] {
        if let Some(variants) = value.get(field).and_then(Value::as_array)
            && variants.len() == 2
        {
            for (i, variant) in variants.iter().enumerate() {
                if variant
                    .as_object()
                    .is_some_and(|o| o.len() == 1 && o.get("type") == Some(&json!("null")))
                {
                    let other = &variants[1 - i];
                    if other.get("type").is_some_and(Value::is_string)
                        && other.get("type") != Some(&json!("null"))
                    {
                        return Some(other);
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn supported_keyword(key: &str) -> bool {
    matches!(
        key,
        "$ref"
            | "type"
            | "nullable"
            | "readOnly"
            | "writeOnly"
            | "deprecated"
            | "properties"
            | "required"
            | "items"
            | "enum"
            | "minimum"
            | "maximum"
            | "exclusiveMinimum"
            | "exclusiveMaximum"
            | "minLength"
            | "maxLength"
            | "minItems"
            | "maxItems"
            | "minProperties"
            | "maxProperties"
            | "uniqueItems"
            | "additionalProperties"
            | "multipleOf"
            | "pattern"
            | "allOf"
            | "oneOf"
            | "anyOf"
            | "description"
            | "summary"
            | "example"
            | "examples"
    )
}

fn visible_property<'a>(mut schema: &'a Value, model: &'a ApiModel, direction: Direction) -> bool {
    let field = if direction == Direction::Request {
        "readOnly"
    } else {
        "writeOnly"
    };
    let mut seen = BTreeSet::new();
    loop {
        let reference = schema.get("$ref").and_then(Value::as_str);
        if (reference.is_none() || model.version == OpenApiVersion::V3_1)
            && schema.get(field) == Some(&Value::Bool(true))
        {
            return false;
        }
        let Some(reference) = reference else {
            return true;
        };
        if !seen.insert(reference) {
            return true;
        }
        let Some(target) = model.root.pointer(reference.trim_start_matches('#')) else {
            return true;
        };
        schema = target;
    }
}
