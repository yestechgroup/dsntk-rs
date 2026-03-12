//! # Tests for TypeScript file parsing

use crate::ts_parser::parse_typescript_source;
use dsntk_feel::FeelType;
use std::path::Path;

#[test]
fn _0001() {
  // Resolves a named interface from TypeScript
  let source = r#"
export interface Applicant {
  age: number;
  name: string;
}
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  let entry = registry.get("Applicant").unwrap();
  assert_eq!(entry.name, "Applicant");
  if let FeelType::Context(fields) = &entry.feel_type {
    assert_eq!(fields.len(), 2);
  } else {
    panic!("expected Context type");
  }
}

#[test]
fn _0002() {
  // Extracts interface fields with correct types
  let source = r#"
export interface Person {
  age: number;
  name: string;
  active: boolean;
}
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  let entry = registry.get("Person").unwrap();
  if let FeelType::Context(fields) = &entry.feel_type {
    let types: Vec<&FeelType> = fields.values().collect();
    assert!(types.contains(&&FeelType::Number));
    assert!(types.contains(&&FeelType::String));
    assert!(types.contains(&&FeelType::Boolean));
  } else {
    panic!("expected Context type");
  }
}

#[test]
fn _0003() {
  // Union type extracts enum values
  let source = r#"
export type Status = "active" | "inactive" | "pending";
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  let entry = registry.get("Status").unwrap();
  assert_eq!(entry.feel_type, FeelType::String);
  let allowed = entry.allowed_values.as_ref().unwrap();
  assert_eq!(allowed.len(), 3);
  assert!(allowed.contains(&"active".to_string()));
  assert!(allowed.contains(&"inactive".to_string()));
  assert!(allowed.contains(&"pending".to_string()));
}

#[test]
fn _0004() {
  // Optional field is marked
  let source = r#"
export interface Config {
  name: string;
  debug?: boolean;
}
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  let entry = registry.get("Config").unwrap();
  assert_eq!(entry.optional_fields.get("debug"), Some(&true));
  assert_eq!(entry.optional_fields.get("name"), Some(&false));
}

#[test]
fn _0005() {
  // Nested object becomes Context type
  let source = r#"
export interface Applicant {
  address: {
    street: string;
    postcode: string;
  };
}
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  let entry = registry.get("Applicant").unwrap();
  if let FeelType::Context(fields) = &entry.feel_type {
    let address_type = fields.values().next().unwrap();
    if let FeelType::Context(addr_fields) = address_type {
      assert_eq!(addr_fields.len(), 2);
    } else {
      panic!("expected nested Context type for address");
    }
  } else {
    panic!("expected Context type");
  }
}

#[test]
fn _0006() {
  // Non-exported types are ignored
  let source = r#"
interface Internal {
  x: number;
}

export interface Public {
  y: string;
}
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  assert!(registry.get("Internal").is_none());
  assert!(registry.get("Public").is_some());
}

#[test]
fn _0007() {
  // Multiple exports from one file
  let source = r#"
export type Status = "a" | "b";
export interface Person {
  name: string;
}
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  assert_eq!(registry.len(), 2);
  assert!(registry.get("Status").is_some());
  assert!(registry.get("Person").is_some());
}

#[test]
fn _0008() {
  // Array type maps to List
  let source = r#"
export interface Data {
  tags: string[];
}
"#;
  let registry = parse_typescript_source(source, Path::new("test.ts")).unwrap();
  let entry = registry.get("Data").unwrap();
  if let FeelType::Context(fields) = &entry.feel_type {
    let tag_type = fields.values().next().unwrap();
    assert!(matches!(tag_type, FeelType::List(_)));
  } else {
    panic!("expected Context type");
  }
}
