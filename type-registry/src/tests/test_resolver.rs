//! # Tests for type resolution and validation

use crate::front_matter::DmnNode;
use crate::registry::{TypeEntry, TypeRegistry, TypeSource};
use crate::resolver::{resolve_field_chain, validate_allowed_value, validate_bkm_signature, validate_link_targets};
use dsntk_feel::{FeelType, Name};
use std::collections::{BTreeMap, HashMap};

fn make_context_type() -> FeelType {
  let mut address_fields = BTreeMap::new();
  address_fields.insert(Name::from("street"), FeelType::String);
  address_fields.insert(Name::from("postcode"), FeelType::String);

  let mut fields = BTreeMap::new();
  fields.insert(Name::from("name"), FeelType::String);
  fields.insert(Name::from("age"), FeelType::Number);
  fields.insert(Name::from("address"), FeelType::Context(address_fields));

  FeelType::Context(fields)
}

// --- Enum/union constraint checking ---

#[test]
fn _0001() {
  // Cell value matches enum passes validation
  let entry = TypeEntry {
    name: "Status".to_string(),
    feel_type: FeelType::String,
    source: TypeSource::Primitive,
    allowed_values: Some(vec!["active".to_string(), "inactive".to_string()]),
    optional_fields: HashMap::new(),
  };
  assert!(validate_allowed_value("active", &entry).is_ok());
}

#[test]
fn _0002() {
  // Cell value violates enum fails validation
  let entry = TypeEntry {
    name: "Status".to_string(),
    feel_type: FeelType::String,
    source: TypeSource::Primitive,
    allowed_values: Some(vec!["active".to_string(), "inactive".to_string()]),
    optional_fields: HashMap::new(),
  };
  let result = validate_allowed_value("unknown", &entry);
  assert!(result.is_err());
  let err_msg = result.unwrap_err().to_string();
  assert!(err_msg.contains("not in allowed values"));
}

#[test]
fn _0003() {
  // No allowed values means any value passes
  let entry = TypeEntry {
    name: "Name".to_string(),
    feel_type: FeelType::String,
    source: TypeSource::Primitive,
    allowed_values: None,
    optional_fields: HashMap::new(),
  };
  assert!(validate_allowed_value("anything", &entry).is_ok());
}

// --- Nested context resolution ---

#[test]
fn _0004() {
  // Nested context resolves field chain
  let ctx = make_context_type();
  let result = resolve_field_chain(&ctx, &["address", "postcode"]);
  assert_eq!(result, Some(FeelType::String));
}

#[test]
fn _0005() {
  // Missing nested field returns None
  let ctx = make_context_type();
  let result = resolve_field_chain(&ctx, &["address", "city"]);
  assert!(result.is_none());
}

#[test]
fn _0006() {
  // Top-level field resolves
  let ctx = make_context_type();
  let result = resolve_field_chain(&ctx, &["name"]);
  assert_eq!(result, Some(FeelType::String));
}

#[test]
fn _0007() {
  // Empty chain returns the type itself
  let ctx = make_context_type();
  let result = resolve_field_chain(&ctx, &[]);
  assert_eq!(result, Some(ctx));
}

// --- BKM signature validation ---

#[test]
fn _0008() {
  // BKM missing return type fails
  let node = DmnNode {
    id: "BKM1".to_string(),
    node_type: "bkm".to_string(),
    name: "Risk Score".to_string(),
    data_type: None,
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
  };
  let result = validate_bkm_signature(&node);
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("missing a return-type"));
}

// --- Link target validation ---

#[test]
fn _0009() {
  // governed-by pointing to knowledge-source is valid
  let decision = DmnNode {
    id: "D1".to_string(),
    node_type: "decision".to_string(),
    name: "Loan Approval".to_string(),
    data_type: None,
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: Some(vec!["lending_policy".to_string()]),
    supported_by: None,
  };
  let ks = DmnNode {
    id: "lending_policy".to_string(),
    node_type: "knowledge-source".to_string(),
    name: "Basel III".to_string(),
    data_type: None,
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
  };
  assert!(validate_link_targets(&decision, &[ks]).is_ok());
}

#[test]
fn _0010() {
  // governed-by pointing to non-knowledge-source is invalid
  let decision = DmnNode {
    id: "D1".to_string(),
    node_type: "decision".to_string(),
    name: "Loan Approval".to_string(),
    data_type: None,
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: Some(vec!["bad_target".to_string()]),
    supported_by: None,
  };
  let bkm = DmnNode {
    id: "bad_target".to_string(),
    node_type: "bkm".to_string(),
    name: "Some BKM".to_string(),
    data_type: None,
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
  };
  let result = validate_link_targets(&decision, &[bkm]);
  assert!(result.is_err());
}
