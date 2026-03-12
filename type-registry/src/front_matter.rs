//! # Front matter parser for Markdown-native DMN files
//!
//! Parses YAML front matter from Markdown files to extract DMN node metadata
//! including type references, BKM signatures, and link declarations.

use crate::errors::*;
use dsntk_common::Result;
use serde::Deserialize;

/// Top-level front matter structure.
#[derive(Debug, Clone, Deserialize)]
pub struct FrontMatter {
  pub dmn: DmnNode,
}

/// A DMN node declared in front matter.
#[derive(Debug, Clone, Deserialize)]
pub struct DmnNode {
  pub id: String,
  #[serde(rename = "type")]
  pub node_type: String,
  pub name: String,
  #[serde(rename = "data-type")]
  pub data_type: Option<DataTypeRef>,
  pub signature: Option<BkmSignature>,
  pub uri: Option<String>,
  pub owner: Option<String>,
  pub requires: Option<Vec<String>>,
  #[serde(rename = "governed-by")]
  pub governed_by: Option<Vec<String>>,
  #[serde(rename = "supported-by")]
  pub supported_by: Option<Vec<String>>,
}

/// A data type reference in front matter.
#[derive(Debug, Clone, Deserialize)]
pub struct DataTypeRef {
  #[serde(rename = "ref")]
  pub type_ref: String,
  pub schema: Option<String>,
}

/// A BKM function signature in front matter.
#[derive(Debug, Clone, Deserialize)]
pub struct BkmSignature {
  pub parameters: Vec<BkmParameter>,
  #[serde(rename = "return-type")]
  pub return_type: Option<String>,
  pub schema: Option<String>,
}

/// A BKM parameter declaration.
#[derive(Debug, Clone, Deserialize)]
pub struct BkmParameter {
  pub name: String,
  #[serde(rename = "type")]
  pub param_type: String,
}

/// The type of DMN node.
#[derive(Debug, Clone, PartialEq)]
pub enum DmnNodeType {
  InputData,
  Decision,
  Bkm,
  KnowledgeSource,
}

impl std::str::FromStr for DmnNodeType {
  type Err = dsntk_common::DsntkError;

  fn from_str(s: &str) -> Result<Self> {
    match s {
      "input-data" => Ok(Self::InputData),
      "decision" => Ok(Self::Decision),
      "bkm" => Ok(Self::Bkm),
      "knowledge-source" => Ok(Self::KnowledgeSource),
      _ => Err(err_invalid_node_type(s)),
    }
  }
}

/// Parses YAML front matter from the beginning of a Markdown file.
/// Front matter is delimited by `---` lines.
pub fn parse_front_matter(content: &str) -> Result<FrontMatter> {
  let trimmed = content.trim_start();
  if !trimmed.starts_with("---") {
    return Err(err_invalid_front_matter("missing opening '---' delimiter"));
  }

  let after_opening = &trimmed[3..];
  let end_pos = after_opening.find("\n---").ok_or_else(|| err_invalid_front_matter("missing closing '---' delimiter"))?;

  let yaml_content = &after_opening[..end_pos];
  let front_matter: FrontMatter = serde_yml::from_str(yaml_content).map_err(|e| err_invalid_front_matter(&e.to_string()))?;

  Ok(front_matter)
}

/// Extracts the body content after the front matter.
pub fn extract_body(content: &str) -> Option<&str> {
  let trimmed = content.trim_start();
  if !trimmed.starts_with("---") {
    return Some(content);
  }
  let after_opening = &trimmed[3..];
  if let Some(end_pos) = after_opening.find("\n---") {
    let after_closing = &after_opening[end_pos + 4..];
    Some(after_closing.trim_start_matches('\n'))
  } else {
    None
  }
}
