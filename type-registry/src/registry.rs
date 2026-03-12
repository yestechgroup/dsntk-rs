//! # Type registry core data structures

use crate::errors::*;
use dsntk_common::Result;
use dsntk_feel::FeelType;
use std::collections::HashMap;
use std::path::PathBuf;

/// Source of a type definition.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeSource {
  /// Type is a built-in FEEL primitive.
  Primitive,
  /// Type was loaded from a TypeScript file.
  TypeScript(PathBuf),
  /// Type was loaded from a JSON Schema file.
  JsonSchema(PathBuf),
}

/// A single type entry in the registry.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeEntry {
  /// The name of the type.
  pub name: String,
  /// The resolved FEEL type.
  pub feel_type: FeelType,
  /// Where this type was defined.
  pub source: TypeSource,
  /// Allowed values (for enums/unions).
  pub allowed_values: Option<Vec<String>>,
  /// Whether specific fields are optional (field name -> optional).
  pub optional_fields: HashMap<String, bool>,
}

/// The type registry holds all resolved types for a DMN project.
#[derive(Debug, Clone, Default)]
pub struct TypeRegistry {
  entries: HashMap<String, TypeEntry>,
}

impl TypeRegistry {
  /// Creates a new empty type registry.
  pub fn new() -> Self {
    Self { entries: HashMap::new() }
  }

  /// Returns the number of types in the registry.
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  /// Returns `true` if the registry contains no types.
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  /// Inserts a type entry into the registry.
  /// Returns an error if a type with the same name already exists from a different source file.
  pub fn insert(&mut self, entry: TypeEntry) -> Result<()> {
    if let Some(existing) = self.entries.get(&entry.name) {
      let existing_path = match &existing.source {
        TypeSource::TypeScript(p) | TypeSource::JsonSchema(p) => p.to_string_lossy().to_string(),
        TypeSource::Primitive => "<primitive>".to_string(),
      };
      let new_path = match &entry.source {
        TypeSource::TypeScript(p) | TypeSource::JsonSchema(p) => p.to_string_lossy().to_string(),
        TypeSource::Primitive => "<primitive>".to_string(),
      };
      if existing_path != new_path {
        return Err(err_ambiguous_type(&entry.name, &existing_path, &new_path));
      }
    }
    self.entries.insert(entry.name.clone(), entry);
    Ok(())
  }

  /// Looks up a type by name.
  pub fn get(&self, name: &str) -> Option<&TypeEntry> {
    self.entries.get(name)
  }

  /// Resolves a type reference to a `FeelType`.
  pub fn resolve(&self, type_name: &str) -> Result<&TypeEntry> {
    self.entries.get(type_name).ok_or_else(|| err_type_not_found(type_name))
  }

  /// Returns an iterator over all type entries.
  pub fn iter(&self) -> impl Iterator<Item = (&String, &TypeEntry)> {
    self.entries.iter()
  }

  /// Merges another registry into this one, checking for ambiguities.
  pub fn merge(&mut self, other: TypeRegistry) -> Result<()> {
    for (_, entry) in other.entries {
      self.insert(entry)?;
    }
    Ok(())
  }
}
