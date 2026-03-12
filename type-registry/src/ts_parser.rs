//! # TypeScript file parser
//!
//! Extracts exported `interface` and `type` declarations from TypeScript files
//! and maps them to FEEL types.

use crate::errors::*;
use crate::registry::{TypeEntry, TypeRegistry, TypeSource};
use dsntk_common::Result;
use dsntk_feel::{FeelType, Name};
use oxc_allocator::Allocator;
use oxc_ast::ast::*;
use oxc_parser::Parser;
use oxc_span::SourceType;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// Parses a TypeScript file and extracts all exported type definitions into a registry.
pub fn parse_typescript_file(path: &Path) -> Result<TypeRegistry> {
  let source_text = std::fs::read_to_string(path).map_err(|e| err_typescript_parse(&path.to_string_lossy(), &e.to_string()))?;
  parse_typescript_source(&source_text, path)
}

/// Parses TypeScript source text and extracts type definitions.
pub fn parse_typescript_source(source_text: &str, path: &Path) -> Result<TypeRegistry> {
  let allocator = Allocator::default();
  let source_type = SourceType::from_path(path).unwrap_or_default();
  let parser_return = Parser::new(&allocator, source_text, source_type).parse();

  if !parser_return.errors.is_empty() {
    let error_messages: Vec<String> = parser_return.errors.iter().map(|e| e.to_string()).collect();
    return Err(err_typescript_parse(&path.to_string_lossy(), &error_messages.join("; ")));
  }

  let mut registry = TypeRegistry::new();
  let path_buf = path.to_path_buf();

  for stmt in &parser_return.program.body {
    if let Statement::ExportNamedDeclaration(export) = stmt {
      if let Some(decl) = &export.declaration {
        match decl {
          Declaration::TSInterfaceDeclaration(iface) => {
            let entry = extract_interface(iface, &path_buf);
            registry.insert(entry)?;
          }
          Declaration::TSTypeAliasDeclaration(alias) => {
            let entry = extract_type_alias(alias, &path_buf);
            registry.insert(entry)?;
          }
          _ => {}
        }
      }
    }
  }

  Ok(registry)
}

/// Extracts a TypeScript interface declaration into a TypeEntry.
fn extract_interface(iface: &TSInterfaceDeclaration, path: &Path) -> TypeEntry {
  let name = iface.id.name.to_string();
  let mut fields = BTreeMap::new();
  let mut optional_fields = HashMap::new();

  for sig in &iface.body.body {
    if let TSSignature::TSPropertySignature(prop) = sig {
      if let Some(field_name_atom) = prop.key.static_name() {
        let field_name = field_name_atom.to_string();
        let feel_type = prop.type_annotation.as_ref().map(|ann| ts_type_to_feel_type(&ann.type_annotation)).unwrap_or(FeelType::Any);
        let is_optional = prop.optional;
        fields.insert(Name::from(field_name.as_str()), feel_type);
        optional_fields.insert(field_name, is_optional);
      }
    }
  }

  TypeEntry {
    name,
    feel_type: FeelType::Context(fields),
    source: TypeSource::TypeScript(path.to_path_buf()),
    allowed_values: None,
    optional_fields,
  }
}

/// Extracts a TypeScript type alias declaration into a TypeEntry.
fn extract_type_alias(alias: &TSTypeAliasDeclaration, path: &Path) -> TypeEntry {
  let name = alias.id.name.to_string();
  let (feel_type, allowed_values) = extract_ts_type_with_values(&alias.type_annotation);

  TypeEntry {
    name,
    feel_type,
    source: TypeSource::TypeScript(path.to_path_buf()),
    allowed_values,
    optional_fields: HashMap::new(),
  }
}

/// Extracts a TypeScript type annotation, returning the FEEL type and optional allowed values.
fn extract_ts_type_with_values(ts_type: &TSType) -> (FeelType, Option<Vec<String>>) {
  if let TSType::TSUnionType(union) = ts_type {
    let mut all_string_literals = true;
    let mut values = Vec::new();
    for member in &union.types {
      if let TSType::TSLiteralType(lit) = member {
        if let TSLiteral::StringLiteral(s) = &lit.literal {
          values.push(s.value.to_string());
        } else {
          all_string_literals = false;
        }
      } else {
        all_string_literals = false;
      }
    }
    if all_string_literals && !values.is_empty() {
      return (FeelType::String, Some(values));
    }
  }
  (ts_type_to_feel_type(ts_type), None)
}

/// Maps a TypeScript type AST node to a FEEL type.
pub fn ts_type_to_feel_type(ts_type: &TSType) -> FeelType {
  match ts_type {
    TSType::TSNumberKeyword(_) => FeelType::Number,
    TSType::TSStringKeyword(_) => FeelType::String,
    TSType::TSBooleanKeyword(_) => FeelType::Boolean,
    TSType::TSNullKeyword(_) => FeelType::Null,
    TSType::TSAnyKeyword(_) => FeelType::Any,
    TSType::TSArrayType(arr) => FeelType::List(Box::new(ts_type_to_feel_type(&arr.element_type))),
    TSType::TSTypeLiteral(literal) => {
      let mut fields = BTreeMap::new();
      for member in &literal.members {
        if let TSSignature::TSPropertySignature(prop) = member {
          if let Some(field_name_atom) = prop.key.static_name() {
            let field_name = field_name_atom.to_string();
            let feel_type = prop.type_annotation.as_ref().map(|ann| ts_type_to_feel_type(&ann.type_annotation)).unwrap_or(FeelType::Any);
            fields.insert(Name::from(field_name.as_str()), feel_type);
          }
        }
      }
      FeelType::Context(fields)
    }
    TSType::TSUnionType(union) => {
      let mut all_string_literals = true;
      for member in &union.types {
        if let TSType::TSLiteralType(lit) = member {
          if !matches!(&lit.literal, TSLiteral::StringLiteral(_)) {
            all_string_literals = false;
          }
        } else {
          all_string_literals = false;
        }
      }
      if all_string_literals {
        FeelType::String
      } else {
        FeelType::Any
      }
    }
    _ => FeelType::Any,
  }
}
