//! # Tests for the type registry core

use crate::registry::{TypeEntry, TypeRegistry, TypeSource};
use dsntk_feel::FeelType;
use std::collections::HashMap;
use std::path::PathBuf;

fn make_entry(name: &str, feel_type: FeelType, source: TypeSource) -> TypeEntry {
  TypeEntry {
    name: name.to_string(),
    feel_type,
    source,
    allowed_values: None,
    optional_fields: HashMap::new(),
  }
}

#[test]
fn _0001() {
  // Empty registry has no types
  let registry = TypeRegistry::new();
  assert!(registry.is_empty());
  assert_eq!(registry.len(), 0);
}

#[test]
fn _0002() {
  // Insert and lookup
  let mut registry = TypeRegistry::new();
  let entry = make_entry("Applicant", FeelType::String, TypeSource::Primitive);
  registry.insert(entry).unwrap();
  assert_eq!(registry.len(), 1);
  assert!(!registry.is_empty());
  let found = registry.get("Applicant");
  assert!(found.is_some());
  assert_eq!(found.unwrap().name, "Applicant");
}

#[test]
fn _0003() {
  // Returns None for unknown type
  let registry = TypeRegistry::new();
  assert!(registry.get("Unknown").is_none());
}

#[test]
fn _0004() {
  // Resolve returns error for unknown type
  let registry = TypeRegistry::new();
  assert!(registry.resolve("Unknown").is_err());
}

#[test]
fn _0005() {
  // Ambiguous type detection from different files
  let mut registry = TypeRegistry::new();
  let entry1 = make_entry("Applicant", FeelType::String, TypeSource::TypeScript(PathBuf::from("a.ts")));
  let entry2 = make_entry("Applicant", FeelType::Number, TypeSource::TypeScript(PathBuf::from("b.ts")));
  registry.insert(entry1).unwrap();
  let result = registry.insert(entry2);
  assert!(result.is_err());
  let err_msg = result.unwrap_err().to_string();
  assert!(err_msg.contains("ambiguous"));
  assert!(err_msg.contains("a.ts"));
  assert!(err_msg.contains("b.ts"));
}

#[test]
fn _0006() {
  // Same type from same file is OK (idempotent)
  let mut registry = TypeRegistry::new();
  let entry1 = make_entry("Applicant", FeelType::String, TypeSource::TypeScript(PathBuf::from("a.ts")));
  let entry2 = make_entry("Applicant", FeelType::Number, TypeSource::TypeScript(PathBuf::from("a.ts")));
  registry.insert(entry1).unwrap();
  registry.insert(entry2).unwrap();
  assert_eq!(registry.len(), 1);
}

#[test]
fn _0007() {
  // Merge registries
  let mut r1 = TypeRegistry::new();
  r1.insert(make_entry("A", FeelType::String, TypeSource::Primitive)).unwrap();
  let mut r2 = TypeRegistry::new();
  r2.insert(make_entry("B", FeelType::Number, TypeSource::Primitive)).unwrap();
  r1.merge(r2).unwrap();
  assert_eq!(r1.len(), 2);
  assert!(r1.get("A").is_some());
  assert!(r1.get("B").is_some());
}

#[test]
fn _0008() {
  // Iterator works
  let mut registry = TypeRegistry::new();
  registry.insert(make_entry("A", FeelType::String, TypeSource::Primitive)).unwrap();
  registry.insert(make_entry("B", FeelType::Number, TypeSource::Primitive)).unwrap();
  let names: Vec<String> = registry.iter().map(|(k, _)| k.clone()).collect();
  assert_eq!(names.len(), 2);
}
