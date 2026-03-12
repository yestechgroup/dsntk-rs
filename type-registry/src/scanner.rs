//! # Folder scanner for type schema files
//!
//! Scans directories for `.ts` and `.json` schema files and builds a unified type registry.

use crate::errors::*;
use crate::registry::TypeRegistry;
use crate::{json_schema_parser, ts_parser};
use dsntk_common::Result;
use std::path::Path;
use walkdir::WalkDir;

/// Scans a directory for `.ts` and `.json` schema files and builds a type registry.
pub fn scan_folder(dir: &Path) -> Result<TypeRegistry> {
  if !dir.exists() {
    return Err(err_schema_file_not_found(&dir.to_string_lossy()));
  }
  if !dir.is_dir() {
    return Err(err_schema_file_not_found(&dir.to_string_lossy()));
  }

  let mut registry = TypeRegistry::new();

  for entry in WalkDir::new(dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
    let path = entry.path();
    if !path.is_file() {
      continue;
    }
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match extension {
      "ts" => {
        let file_registry = ts_parser::parse_typescript_file(path)?;
        registry.merge(file_registry)?;
      }
      "json" => {
        let file_registry = json_schema_parser::parse_json_schema_file(path)?;
        registry.merge(file_registry)?;
      }
      _ => {}
    }
  }

  Ok(registry)
}

/// Resolves a schema reference which can be either a file path or directory path.
pub fn resolve_schema(schema_path: &str, base_dir: &Path) -> Result<TypeRegistry> {
  let path = base_dir.join(schema_path);
  if path.is_dir() {
    scan_folder(&path)
  } else if path.is_file() {
    let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match extension {
      "ts" => ts_parser::parse_typescript_file(&path),
      "json" => json_schema_parser::parse_json_schema_file(&path),
      _ => Err(err_schema_file_not_found(&path.to_string_lossy())),
    }
  } else {
    Err(err_schema_file_not_found(&path.to_string_lossy()))
  }
}
