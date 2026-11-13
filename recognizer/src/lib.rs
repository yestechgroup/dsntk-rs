//! # Decision table recognizer
//!
//! Recognizes decision tables defined as plain text.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(rustdoc::missing_crate_level_docs)]
// The recognizer's table traversal walks a character grid by row and
// column indexes — the bounded index loops read clearest as indexes.
#![allow(clippy::needless_range_loop)]

#[macro_use]
extern crate dsntk_macros;

mod builder;
mod canvas;
mod errors;
mod markdown;
mod model;
mod plane;
mod point;
mod recognizer;
mod rect;
mod tests;
mod utils;

pub use builder::from_unicode;
pub use markdown::from_markdown;

pub use model::{
  AnnotationClause, AnnotationEntry, BuiltinAggregator, DecisionRule, DecisionTable, DecisionTableOrientation, HitPolicy, InputClause, InputEntry, OutputClause, OutputEntry,
};
