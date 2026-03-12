//! # DMN XML export
//!
//! Converts resolved types from the type registry to DMN `ItemDefinition` XML elements.

use crate::registry::TypeEntry;
use dsntk_feel::FeelType;

/// Exports a type entry to DMN ItemDefinition XML.
pub fn type_entry_to_item_definition_xml(entry: &TypeEntry, indent: usize) -> String {
  let prefix = " ".repeat(indent);
  let mut xml = String::new();

  xml.push_str(&format!("{prefix}<itemDefinition name=\"{}\"", entry.name));

  match &entry.feel_type {
    FeelType::Context(_) => {
      xml.push_str(">\n");
      xml.push_str(&feel_type_to_item_components(&entry.feel_type, &entry.name, indent + 2));
      if let Some(ref allowed) = entry.allowed_values {
        xml.push_str(&allowed_values_to_xml(allowed, indent + 2));
      }
      xml.push_str(&format!("{prefix}</itemDefinition>\n"));
    }
    _ => {
      let type_ref = feel_type_to_type_ref(&entry.feel_type);
      if !type_ref.is_empty() {
        xml.push_str(&format!(" typeRef=\"{type_ref}\""));
      }
      if let Some(ref allowed) = entry.allowed_values {
        xml.push_str(">\n");
        xml.push_str(&allowed_values_to_xml(allowed, indent + 2));
        xml.push_str(&format!("{prefix}</itemDefinition>\n"));
      } else {
        xml.push_str("/>\n");
      }
    }
  }

  xml
}

/// Converts a FEEL type to the DMN typeRef string.
fn feel_type_to_type_ref(feel_type: &FeelType) -> String {
  match feel_type {
    FeelType::Number => "number".to_string(),
    FeelType::String => "string".to_string(),
    FeelType::Boolean => "boolean".to_string(),
    FeelType::Date => "date".to_string(),
    FeelType::DateTime => "dateTime".to_string(),
    FeelType::Time => "time".to_string(),
    FeelType::DaysAndTimeDuration => "dayTimeDuration".to_string(),
    FeelType::YearsAndMonthsDuration => "yearMonthDuration".to_string(),
    FeelType::Any => "Any".to_string(),
    _ => String::new(),
  }
}

/// Converts context fields to itemComponent XML elements.
fn feel_type_to_item_components(feel_type: &FeelType, _parent_name: &str, indent: usize) -> String {
  let prefix = " ".repeat(indent);
  let mut xml = String::new();

  if let FeelType::Context(entries) = feel_type {
    for (name, field_type) in entries {
      let field_name = name.to_string();
      match field_type {
        FeelType::Context(_) => {
          xml.push_str(&format!("{prefix}<itemComponent name=\"{field_name}\">\n"));
          xml.push_str(&feel_type_to_item_components(field_type, &field_name, indent + 2));
          xml.push_str(&format!("{prefix}</itemComponent>\n"));
        }
        _ => {
          let type_ref = feel_type_to_type_ref(field_type);
          xml.push_str(&format!("{prefix}<itemComponent name=\"{field_name}\" typeRef=\"{type_ref}\"/>\n"));
        }
      }
    }
  }

  xml
}

/// Converts allowed values to DMN allowedValues XML.
fn allowed_values_to_xml(values: &[String], indent: usize) -> String {
  let prefix = " ".repeat(indent);
  let constraints: Vec<String> = values.iter().map(|v| format!("\"{v}\"")).collect();
  format!("{prefix}<allowedValues><text>{}</text></allowedValues>\n", constraints.join(","))
}

/// Generates a complete DMN Definitions XML fragment with ItemDefinition elements.
pub fn registry_to_item_definitions_xml(entries: &[&TypeEntry], indent: usize) -> String {
  let mut xml = String::new();
  for entry in entries {
    xml.push_str(&type_entry_to_item_definition_xml(entry, indent));
  }
  xml
}
