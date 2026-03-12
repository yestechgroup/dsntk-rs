//! # dsntk-type-registry
//!
//! Type registry for Markdown-native DMN. Resolves type definitions from
//! TypeScript and JSON Schema files referenced in DMN node front matter.

#[macro_use]
extern crate dsntk_macros;

pub mod errors;
pub mod exporter;
pub mod front_matter;
pub mod json_schema_parser;
pub mod primitive;
pub mod registry;
pub mod resolver;
pub mod scanner;
pub mod ts_parser;

#[cfg(test)]
mod tests;

pub use front_matter::{parse_front_matter, BkmParameter, BkmSignature, DataTypeRef, DmnNode, DmnNodeType, FrontMatter};
pub use registry::{TypeEntry, TypeRegistry, TypeSource};
