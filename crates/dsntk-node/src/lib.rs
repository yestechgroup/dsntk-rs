//! # dsntk-node
//!
//! napi-rs bridge exposing dsntk Rust evaluation engine to Node.js.
//! Used by the Tauri + SvelteKit desktop application.
//!
//! Operates on **markdown-native DMN projects** (created via `dsntk new`),
//! NOT XML. Each project is a directory containing:
//! - `decisions/*.md` files with YAML front matter and markdown decision tables
//! - `types/*.ts` files with TypeScript type definitions

use dsntk_feel::values::Value;
use dsntk_type_registry::parse_front_matter;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use walkdir::WalkDir;

/// A node in the DMN flow graph for SvelteFlow visualization.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
  /// The `dmn.id` from front matter.
  pub id: String,
  /// The `dmn.name` from front matter.
  pub label: String,
  /// The node type: "inputData", "decision", "bkm", "knowledgeSource".
  pub node_type: String,
  /// Optional type reference (e.g. "ApplicantData").
  pub data_type_ref: Option<String>,
  /// Optional schema path (e.g. "../types/loan.ts").
  pub schema_path: Option<String>,
  /// The markdown body content (documentation + decision table).
  pub body: String,
  /// The source file path relative to the project root.
  pub source_file: String,
}

/// An edge in the DMN flow graph for SvelteFlow visualization.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
  pub id: String,
  pub source: String,
  pub target: String,
  /// "requires" or "governed-by".
  pub edge_type: String,
}

/// The complete flow graph for SvelteFlow rendering.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowGraph {
  pub nodes: Vec<FlowNode>,
  pub edges: Vec<FlowEdge>,
  /// Project directory name.
  pub project_name: String,
  /// TypeScript type definitions found in the project.
  pub type_files: Vec<TypeFile>,
}

/// A TypeScript type file from the project.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeFile {
  /// Path relative to project root.
  pub path: String,
  /// File content.
  pub content: String,
}

/// Trace result for a single decision table row.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowTrace {
  pub index: u32,
  pub outcome: String,
}

/// Evaluation trace for the entire model.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationTrace {
  pub node_results: HashMap<String, NodeTrace>,
  pub output_value: String,
}

/// Trace result for a single node.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTrace {
  /// "hit", "miss", "ignored", or "pending".
  pub status: String,
  /// The evaluated value as a string.
  pub value: String,
  /// Row-level traces for decision table nodes.
  pub table_traces: Vec<RowTrace>,
}

/// Parsed decision table info for a decision node.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTableInfo {
  pub node_id: String,
  pub hit_policy: String,
  pub input_columns: Vec<String>,
  pub output_columns: Vec<String>,
  pub rules: Vec<DecisionRuleInfo>,
}

/// A single rule row from a decision table.
#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRuleInfo {
  pub index: u32,
  pub input_entries: Vec<String>,
  pub output_entries: Vec<String>,
}

/// Loads a markdown DMN project from a directory and returns a FlowGraph.
///
/// The directory should contain `decisions/*.md` files with YAML front matter
/// (created via `dsntk new`). This does NOT use XML.
#[napi]
pub fn load_dmn_project(project_dir: String) -> Result<FlowGraph> {
  let project_path = Path::new(&project_dir);
  if !project_path.is_dir() {
    return Err(Error::from_reason(format!("'{}' is not a directory", project_dir)));
  }

  let project_name = project_path
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_else(|| "unknown".to_string());

  let mut nodes = Vec::new();
  let mut edges = Vec::new();
  let mut type_files = Vec::new();
  let mut edge_counter = 0u32;

  // Scan for all .md files in the project directory
  for entry in WalkDir::new(project_path).into_iter().filter_map(|e| e.ok()) {
    let path = entry.path();
    let relative_path = path.strip_prefix(project_path).unwrap_or(path).to_string_lossy().to_string();

    if path.is_file() {
      let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

      if ext == "md" && relative_path != "README.md" {
        // Parse markdown DMN file
        let content = std::fs::read_to_string(path).map_err(|e| Error::from_reason(format!("Failed to read '{}': {}", relative_path, e)))?;

        // Try to parse front matter — skip files without valid DMN front matter
        let front_matter = match parse_front_matter(&content) {
          Ok(fm) => fm,
          Err(_) => continue,
        };

        let body = dsntk_type_registry::front_matter::extract_body(&content).unwrap_or("").to_string();

        let node_type = match front_matter.dmn.node_type.as_str() {
          "input-data" => "inputData",
          "decision" => "decision",
          "bkm" => "bkm",
          "knowledge-source" => "knowledgeSource",
          _ => "decision",
        };

        nodes.push(FlowNode {
          id: front_matter.dmn.id.clone(),
          label: front_matter.dmn.name.clone(),
          node_type: node_type.to_string(),
          data_type_ref: front_matter.dmn.data_type.as_ref().map(|dt| dt.type_ref.clone()),
          schema_path: front_matter.dmn.data_type.as_ref().and_then(|dt| dt.schema.clone()),
          body,
          source_file: relative_path,
        });

        // Build "requires" edges
        if let Some(requires) = &front_matter.dmn.requires {
          for required_id in requires {
            edges.push(FlowEdge {
              id: format!("e{}", edge_counter),
              source: required_id.clone(),
              target: front_matter.dmn.id.clone(),
              edge_type: "requires".to_string(),
            });
            edge_counter += 1;
          }
        }

        // Build "governed-by" edges
        if let Some(governed_by) = &front_matter.dmn.governed_by {
          for gov_id in governed_by {
            edges.push(FlowEdge {
              id: format!("e{}", edge_counter),
              source: gov_id.clone(),
              target: front_matter.dmn.id.clone(),
              edge_type: "governed-by".to_string(),
            });
            edge_counter += 1;
          }
        }
      } else if ext == "ts" {
        // Collect TypeScript type definition files
        let content = std::fs::read_to_string(path).map_err(|e| Error::from_reason(format!("Failed to read '{}': {}", relative_path, e)))?;
        type_files.push(TypeFile {
          path: relative_path,
          content,
        });
      }
    }
  }

  Ok(FlowGraph {
    nodes,
    edges,
    project_name,
    type_files,
  })
}

/// Parses a decision table from a markdown DMN file and returns structured info.
#[napi]
pub fn parse_decision_table(file_path: String) -> Result<DecisionTableInfo> {
  let content = std::fs::read_to_string(&file_path).map_err(|e| Error::from_reason(format!("Failed to read '{}': {}", file_path, e)))?;

  let front_matter = parse_front_matter(&content).map_err(|e| Error::from_reason(format!("Failed to parse front matter: {}", e)))?;

  let body = dsntk_type_registry::front_matter::extract_body(&content).unwrap_or("");

  let dt = dsntk_recognizer::from_markdown(body, false).map_err(|e| Error::from_reason(format!("Failed to parse decision table: {}", e)))?;

  let input_columns: Vec<String> = dt.input_clauses.iter().map(|ic| ic.input_expression.clone()).collect();

  let output_columns: Vec<String> = dt
    .output_clauses
    .iter()
    .map(|oc| oc.output_component_name.clone().unwrap_or_default())
    .collect();

  let rules: Vec<DecisionRuleInfo> = dt
    .rules
    .iter()
    .enumerate()
    .map(|(i, rule)| {
      let input_entries: Vec<String> = rule.input_entries.iter().map(|ie| ie.text.clone()).collect();
      let output_entries: Vec<String> = rule.output_entries.iter().map(|oe| oe.text.clone()).collect();
      DecisionRuleInfo {
        index: i as u32,
        input_entries,
        output_entries,
      }
    })
    .collect();

  Ok(DecisionTableInfo {
    node_id: front_matter.dmn.id,
    hit_policy: format!("{}", dt.hit_policy),
    input_columns,
    output_columns,
    rules,
  })
}

/// Evaluates a FEEL expression and returns the result as a string.
#[napi]
pub fn evaluate_feel_expression(expression: String, context_json: String) -> Result<String> {
  let scope = dsntk_feel::FeelScope::default();
  let node = dsntk_feel_parser::parse_expression(&scope, &expression, false).map_err(|e| Error::from_reason(format!("Failed to parse FEEL expression: {}", e)))?;

  let input_value: serde_json::Value =
    serde_json::from_str(&context_json).map_err(|e| Error::from_reason(format!("Failed to parse context JSON: {}", e)))?;
  let context = json_to_feel_context(&input_value);
  let eval_scope: dsntk_feel::FeelScope = context.into();
  let result = dsntk_feel_evaluator::evaluate(&eval_scope, &node);

  Ok(format!("{}", result))
}

/// Converts a JSON value to a FEEL context.
fn json_to_feel_context(value: &serde_json::Value) -> dsntk_feel::context::FeelContext {
  let mut ctx = dsntk_feel::context::FeelContext::default();
  if let serde_json::Value::Object(map) = value {
    for (key, val) in map {
      let name = dsntk_feel::Name::from(key.as_str());
      ctx.set_entry(&name, json_to_feel_value(val));
    }
  }
  ctx
}

/// Converts a JSON value to a FEEL value.
fn json_to_feel_value(value: &serde_json::Value) -> Value {
  match value {
    serde_json::Value::Null => dsntk_feel::value_null!(),
    serde_json::Value::Bool(b) => Value::Boolean(*b),
    serde_json::Value::Number(n) => {
      if let Ok(num) = n.to_string().parse::<dsntk_feel::FeelNumber>() {
        Value::Number(num)
      } else {
        dsntk_feel::value_null!()
      }
    }
    serde_json::Value::String(s) => Value::String(s.clone()),
    serde_json::Value::Array(arr) => {
      let items: Vec<Value> = arr.iter().map(json_to_feel_value).collect();
      Value::List(items)
    }
    serde_json::Value::Object(map) => {
      let mut ctx = dsntk_feel::context::FeelContext::default();
      for (key, val) in map {
        let name = dsntk_feel::Name::from(key.as_str());
        ctx.set_entry(&name, json_to_feel_value(val));
      }
      Value::Context(ctx)
    }
  }
}
