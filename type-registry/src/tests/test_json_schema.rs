//! # Tests for JSON Schema parsing

use crate::json_schema_parser::parse_json_schema_source;
use dsntk_feel::FeelType;
use std::path::Path;

#[test]
fn _0001() {
  // Resolves by $id
  let source = r#"{ "$id": "Applicant", "type": "object", "properties": { "age": { "type": "number" } }, "required": ["age"] }"#;
  let registry = parse_json_schema_source(source, Path::new("test.json")).unwrap();
  let entry = registry.get("Applicant").unwrap();
  assert_eq!(entry.name, "Applicant");
}

#[test]
fn _0002() {
  // Resolves by title fallback
  let source = r#"{ "title": "CreditScore", "type": "number" }"#;
  let registry = parse_json_schema_source(source, Path::new("test.json")).unwrap();
  let entry = registry.get("CreditScore").unwrap();
  assert_eq!(entry.feel_type, FeelType::Number);
}

#[test]
fn _0003() {
  // Object maps to Context type
  let source = r#"{
    "$id": "Person",
    "type": "object",
    "properties": {
      "name": { "type": "string" },
      "age": { "type": "number" }
    },
    "required": ["name", "age"]
  }"#;
  let registry = parse_json_schema_source(source, Path::new("test.json")).unwrap();
  let entry = registry.get("Person").unwrap();
  if let FeelType::Context(fields) = &entry.feel_type {
    assert_eq!(fields.len(), 2);
  } else {
    panic!("expected Context type");
  }
}

#[test]
fn _0004() {
  // Enum extracts allowed values
  let source = r#"{
    "$id": "Status",
    "type": "string",
    "enum": ["active", "inactive", "pending"]
  }"#;
  let registry = parse_json_schema_source(source, Path::new("test.json")).unwrap();
  let entry = registry.get("Status").unwrap();
  let allowed = entry.allowed_values.as_ref().unwrap();
  assert_eq!(allowed.len(), 3);
  assert!(allowed.contains(&"active".to_string()));
}

#[test]
fn _0005() {
  // Required vs optional fields
  let source = r#"{
    "$id": "Config",
    "type": "object",
    "properties": {
      "name": { "type": "string" },
      "debug": { "type": "boolean" }
    },
    "required": ["name"]
  }"#;
  let registry = parse_json_schema_source(source, Path::new("test.json")).unwrap();
  let entry = registry.get("Config").unwrap();
  assert_eq!(entry.optional_fields.get("name"), Some(&false));
  assert_eq!(entry.optional_fields.get("debug"), Some(&true));
}

#[test]
fn _0006() {
  // Missing $id and title errors
  let source = r#"{ "type": "object", "properties": {} }"#;
  let result = parse_json_schema_source(source, Path::new("test.json"));
  assert!(result.is_err());
}

#[test]
fn _0007() {
  // Boolean type
  let source = r#"{ "$id": "Flag", "type": "boolean" }"#;
  let registry = parse_json_schema_source(source, Path::new("test.json")).unwrap();
  let entry = registry.get("Flag").unwrap();
  assert_eq!(entry.feel_type, FeelType::Boolean);
}

#[test]
fn _0008() {
  // Array type maps to List
  let source = r#"{ "$id": "Tags", "type": "array", "items": { "type": "string" } }"#;
  let registry = parse_json_schema_source(source, Path::new("test.json")).unwrap();
  let entry = registry.get("Tags").unwrap();
  assert!(matches!(entry.feel_type, FeelType::List(_)));
}
