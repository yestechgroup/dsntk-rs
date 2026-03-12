//! # Tests for DMN XML export

use crate::exporter::{registry_to_item_definitions_xml, type_entry_to_item_definition_xml};
use crate::registry::{TypeEntry, TypeSource};
use dsntk_feel::{FeelType, Name};
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

#[test]
fn _0001() {
  // Simple type exports to ItemDefinition XML
  let entry = TypeEntry {
    name: "CreditScore".to_string(),
    feel_type: FeelType::Number,
    source: TypeSource::Primitive,
    allowed_values: None,
    optional_fields: HashMap::new(),
  };
  let xml = type_entry_to_item_definition_xml(&entry, 0);
  assert!(xml.contains("itemDefinition"));
  assert!(xml.contains("CreditScore"));
  assert!(xml.contains("typeRef=\"number\""));
}

#[test]
fn _0002() {
  // Enum type exports with allowed values
  let entry = TypeEntry {
    name: "Status".to_string(),
    feel_type: FeelType::String,
    source: TypeSource::TypeScript(PathBuf::from("test.ts")),
    allowed_values: Some(vec!["active".to_string(), "inactive".to_string()]),
    optional_fields: HashMap::new(),
  };
  let xml = type_entry_to_item_definition_xml(&entry, 0);
  assert!(xml.contains("allowedValues"));
  assert!(xml.contains("\"active\""));
  assert!(xml.contains("\"inactive\""));
}

#[test]
fn _0003() {
  // Context type exports with item components
  let mut fields = BTreeMap::new();
  fields.insert(Name::from("age"), FeelType::Number);
  fields.insert(Name::from("name"), FeelType::String);

  let entry = TypeEntry {
    name: "Person".to_string(),
    feel_type: FeelType::Context(fields),
    source: TypeSource::TypeScript(PathBuf::from("test.ts")),
    allowed_values: None,
    optional_fields: HashMap::new(),
  };
  let xml = type_entry_to_item_definition_xml(&entry, 0);
  assert!(xml.contains("itemComponent"));
  assert!(xml.contains("age"));
  assert!(xml.contains("name"));
  assert!(xml.contains("typeRef=\"number\""));
  assert!(xml.contains("typeRef=\"string\""));
}

#[test]
fn _0004() {
  // Nested type exports recursively
  let mut address_fields = BTreeMap::new();
  address_fields.insert(Name::from("street"), FeelType::String);

  let mut fields = BTreeMap::new();
  fields.insert(Name::from("address"), FeelType::Context(address_fields));

  let entry = TypeEntry {
    name: "Applicant".to_string(),
    feel_type: FeelType::Context(fields),
    source: TypeSource::TypeScript(PathBuf::from("test.ts")),
    allowed_values: None,
    optional_fields: HashMap::new(),
  };
  let xml = type_entry_to_item_definition_xml(&entry, 0);
  assert!(xml.contains("itemComponent name=\"address\""));
  assert!(xml.contains("itemComponent name=\"street\""));
}

#[test]
fn _0005() {
  // Multiple entries export together
  let e1 = TypeEntry {
    name: "A".to_string(),
    feel_type: FeelType::Number,
    source: TypeSource::Primitive,
    allowed_values: None,
    optional_fields: HashMap::new(),
  };
  let e2 = TypeEntry {
    name: "B".to_string(),
    feel_type: FeelType::String,
    source: TypeSource::Primitive,
    allowed_values: None,
    optional_fields: HashMap::new(),
  };
  let xml = registry_to_item_definitions_xml(&[&e1, &e2], 0);
  assert!(xml.contains("name=\"A\""));
  assert!(xml.contains("name=\"B\""));
}
