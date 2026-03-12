//! # JSON Schema file parser
//!
//! Extracts type definitions from JSON Schema files and maps them to FEEL types.

use crate::errors::*;
use crate::registry::{TypeEntry, TypeRegistry, TypeSource};
use dsntk_common::Result;
use dsntk_feel::{FeelType, Name};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Parses a JSON Schema file and extracts type definitions into a registry.
pub fn parse_json_schema_file(path: &Path) -> Result<TypeRegistry> {
  let content = std::fs::read_to_string(path).map_err(|e| err_json_schema_parse(&path.to_string_lossy(), &e.to_string()))?;
  parse_json_schema_source(&content, path)
}

/// Parses JSON Schema source text and extracts type definitions.
pub fn parse_json_schema_source(source: &str, path: &Path) -> Result<TypeRegistry> {
  let value: serde_json::Value = serde_json::from_str(source).map_err(|e| err_json_schema_parse(&path.to_string_lossy(), &e.to_string()))?;
  let mut registry = TypeRegistry::new();

  // Determine the type name from $id or title
  let type_name = value
    .get("$id")
    .and_then(|v| v.as_str())
    .or_else(|| value.get("title").and_then(|v| v.as_str()))
    .unwrap_or("")
    .to_string();

  if type_name.is_empty() {
    return Err(err_json_schema_parse(&path.to_string_lossy(), "schema must have '$id' or 'title' field"));
  }

  let entry = json_schema_to_type_entry(&type_name, &value, path);
  registry.insert(entry)?;

  Ok(registry)
}

/// Converts a JSON Schema value into a TypeEntry.
fn json_schema_to_type_entry(name: &str, schema: &serde_json::Value, path: &Path) -> TypeEntry {
  let schema_type = schema.get("type").and_then(|v| v.as_str()).unwrap_or("");
  let (feel_type, allowed_values, optional_fields) = match schema_type {
    "object" => {
      let (ctx_type, opt_fields) = json_schema_object_to_context(schema);
      (ctx_type, None, opt_fields)
    }
    "string" => {
      let allowed = schema
        .get("enum")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect());
      (FeelType::String, allowed, HashMap::new())
    }
    "number" | "integer" => (FeelType::Number, None, HashMap::new()),
    "boolean" => (FeelType::Boolean, None, HashMap::new()),
    "array" => {
      let item_type = schema.get("items").map(json_schema_type_to_feel_type).unwrap_or(FeelType::Any);
      (FeelType::List(Box::new(item_type)), None, HashMap::new())
    }
    _ => (FeelType::Any, None, HashMap::new()),
  };

  TypeEntry {
    name: name.to_string(),
    feel_type,
    source: TypeSource::JsonSchema(path.to_path_buf()),
    allowed_values,
    optional_fields,
  }
}

/// Converts a JSON Schema object type to a FEEL Context type.
fn json_schema_object_to_context(schema: &serde_json::Value) -> (FeelType, HashMap<String, bool>) {
  let mut fields = BTreeMap::new();
  let mut optional_fields = HashMap::new();

  let required: Vec<String> = schema
    .get("required")
    .and_then(|v| v.as_array())
    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
    .unwrap_or_default();

  if let Some(properties) = schema.get("properties").and_then(|v| v.as_object()) {
    for (field_name, field_schema) in properties {
      let feel_type = json_schema_type_to_feel_type(field_schema);
      let is_optional = !required.contains(field_name);
      fields.insert(Name::from(field_name.as_str()), feel_type);
      optional_fields.insert(field_name.clone(), is_optional);
    }
  }

  (FeelType::Context(fields), optional_fields)
}

/// Maps a JSON Schema type to a FEEL type.
fn json_schema_type_to_feel_type(schema: &serde_json::Value) -> FeelType {
  let schema_type = schema.get("type").and_then(|v| v.as_str()).unwrap_or("");
  match schema_type {
    "string" => FeelType::String,
    "number" | "integer" => FeelType::Number,
    "boolean" => FeelType::Boolean,
    "object" => {
      let (ctx_type, _) = json_schema_object_to_context(schema);
      ctx_type
    }
    "array" => {
      let item_type = schema.get("items").map(json_schema_type_to_feel_type).unwrap_or(FeelType::Any);
      FeelType::List(Box::new(item_type))
    }
    _ => FeelType::Any,
  }
}
