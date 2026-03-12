//! # Type resolver
//!
//! Resolves type references from DMN node front matter using the type registry.

use crate::errors::*;
use crate::front_matter::{DataTypeRef, DmnNode};
use crate::registry::{TypeEntry, TypeRegistry};
use crate::{primitive, scanner};
use dsntk_common::Result;
use dsntk_feel::FeelType;
use std::path::Path;

/// Resolves a data-type reference from front matter.
/// Handles primitives (no schema needed), file references, and folder references.
pub fn resolve_data_type(data_type: &DataTypeRef, base_dir: &Path, registry: &TypeRegistry) -> Result<TypeEntry> {
  let type_name = &data_type.type_ref;

  // Try primitive first
  if let Some(entry) = primitive::resolve_primitive(type_name) {
    return Ok(entry);
  }

  // If a schema path is specified, resolve from it
  if let Some(schema_path) = &data_type.schema {
    let schema_registry = scanner::resolve_schema(schema_path, base_dir)?;
    return schema_registry
      .resolve(type_name)
      .cloned()
      .map_err(|_| err_type_not_found_in_schema(type_name, schema_path));
  }

  // Try the main registry
  registry.resolve(type_name).cloned()
}

/// Validates that a value is within the allowed values of a type entry.
pub fn validate_allowed_value(value: &str, entry: &TypeEntry) -> Result<()> {
  if let Some(ref allowed) = entry.allowed_values {
    if !allowed.iter().any(|v| v == value) {
      return Err(err_enum_violation(value, allowed));
    }
  }
  Ok(())
}

/// Resolves a nested field chain (e.g., "address.postcode") through a Context type.
pub fn resolve_field_chain(feel_type: &FeelType, field_chain: &[&str]) -> Option<FeelType> {
  if field_chain.is_empty() {
    return Some(feel_type.clone());
  }

  if let FeelType::Context(entries) = feel_type {
    let field_name = field_chain[0];
    for (name, field_type) in entries {
      if name.to_string() == field_name {
        return resolve_field_chain(field_type, &field_chain[1..]);
      }
    }
  }

  None
}

/// Validates that a DMN node's links point to the correct node types.
pub fn validate_link_targets(node: &DmnNode, nodes: &[DmnNode]) -> Result<()> {
  // Validate governed-by links point to knowledge-source nodes
  if let Some(ref governed_by) = node.governed_by {
    for target_path in governed_by {
      if let Some(target_node) = find_node_by_path(target_path, nodes) {
        if target_node.node_type != "knowledge-source" {
          return Err(err_invalid_link_target("governed-by", target_path, "knowledge-source"));
        }
      }
    }
  }

  // Validate supported-by links point to BKM nodes
  if let Some(ref supported_by) = node.supported_by {
    for target_path in supported_by {
      if let Some(target_node) = find_node_by_path(target_path, nodes) {
        if target_node.node_type != "bkm" {
          return Err(err_invalid_link_target("supported-by", target_path, "bkm"));
        }
      }
    }
  }

  Ok(())
}

/// Finds a node by its file path reference.
fn find_node_by_path<'a>(path: &str, nodes: &'a [DmnNode]) -> Option<&'a DmnNode> {
  // Simple matching by checking if the path ends with a known node ID or name
  nodes.iter().find(|n| path.contains(&n.id) || path.contains(&n.name))
}

/// Validates a BKM node's signature.
pub fn validate_bkm_signature(node: &DmnNode) -> Result<()> {
  if node.node_type != "bkm" {
    return Ok(());
  }
  if let Some(ref sig) = node.signature {
    if sig.return_type.is_none() {
      return Err(err_missing_return_type(&node.name));
    }
  } else {
    return Err(err_missing_return_type(&node.name));
  }
  Ok(())
}
