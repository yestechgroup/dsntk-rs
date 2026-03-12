//! # Tests for primitive type resolution

use crate::primitive::{is_primitive, resolve_primitive};
use crate::registry::TypeSource;
use dsntk_feel::FeelType;

#[test]
fn _0001() {
  // number resolves to FeelType::Number
  let entry = resolve_primitive("number").unwrap();
  assert_eq!(entry.feel_type, FeelType::Number);
  assert_eq!(entry.source, TypeSource::Primitive);
}

#[test]
fn _0002() {
  // string resolves to FeelType::String
  let entry = resolve_primitive("string").unwrap();
  assert_eq!(entry.feel_type, FeelType::String);
}

#[test]
fn _0003() {
  // boolean resolves to FeelType::Boolean
  let entry = resolve_primitive("boolean").unwrap();
  assert_eq!(entry.feel_type, FeelType::Boolean);
}

#[test]
fn _0004() {
  // date resolves to FeelType::Date
  let entry = resolve_primitive("date").unwrap();
  assert_eq!(entry.feel_type, FeelType::Date);
}

#[test]
fn _0005() {
  // Unknown type returns None
  assert!(resolve_primitive("FooBar").is_none());
}

#[test]
fn _0006() {
  // is_primitive recognizes all primitives
  assert!(is_primitive("number"));
  assert!(is_primitive("string"));
  assert!(is_primitive("boolean"));
  assert!(is_primitive("date"));
  assert!(is_primitive("time"));
  assert!(!is_primitive("Applicant"));
  assert!(!is_primitive(""));
}

#[test]
fn _0007() {
  // Primitive entry has no allowed values
  let entry = resolve_primitive("number").unwrap();
  assert!(entry.allowed_values.is_none());
}

#[test]
fn _0008() {
  // All temporal types resolve
  assert!(resolve_primitive("date and time").is_some());
  assert!(resolve_primitive("days and time duration").is_some());
  assert!(resolve_primitive("years and months duration").is_some());
  assert!(resolve_primitive("time").is_some());
}
