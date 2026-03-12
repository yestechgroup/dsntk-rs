//! # DSNTK Visual DMN Explorer
//!
//! Tauri desktop application for visualizing markdown-native DMN projects.
//! Operates on projects created via `dsntk new` — no XML involved.

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use dsntk_type_registry::parse_front_matter;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A node in the DMN flow graph for SvelteFlow visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowNode {
  pub id: String,
  pub label: String,
  pub node_type: String,
  pub data_type_ref: Option<String>,
  pub schema_path: Option<String>,
  pub body: String,
  pub source_file: String,
}

/// An edge in the DMN flow graph for SvelteFlow visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowEdge {
  pub id: String,
  pub source: String,
  pub target: String,
  pub edge_type: String,
}

/// The complete flow graph for SvelteFlow rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowGraph {
  pub nodes: Vec<FlowNode>,
  pub edges: Vec<FlowEdge>,
  pub project_name: String,
  pub type_files: Vec<TypeFile>,
}

/// A TypeScript type file from the project.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeFile {
  pub path: String,
  pub content: String,
}

/// Parsed decision table info for a decision node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionTableInfo {
  pub node_id: String,
  pub hit_policy: String,
  pub input_columns: Vec<String>,
  pub output_columns: Vec<String>,
  pub rules: Vec<DecisionRuleInfo>,
}

/// A single rule row from a decision table.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionRuleInfo {
  pub index: u32,
  pub input_entries: Vec<String>,
  pub output_entries: Vec<String>,
}

/// Tauri command: Load a markdown DMN project directory and return the flow graph.
///
/// Scans `decisions/*.md` files with YAML front matter and `types/*.ts` files.
/// Does NOT use XML.
#[tauri::command]
fn load_dmn_project(project_dir: String) -> Result<FlowGraph, String> {
  let project_path = Path::new(&project_dir);
  if !project_path.is_dir() {
    return Err(format!("'{}' is not a directory", project_dir));
  }

  let project_name = project_path
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_else(|| "unknown".to_string());

  let mut nodes = Vec::new();
  let mut edges = Vec::new();
  let mut type_files = Vec::new();
  let mut edge_counter = 0u32;

  for entry in WalkDir::new(project_path).into_iter().filter_map(|e| e.ok()) {
    let path = entry.path();
    let relative_path = path.strip_prefix(project_path).unwrap_or(path).to_string_lossy().to_string();

    if path.is_file() {
      let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

      if ext == "md" && relative_path != "README.md" {
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", relative_path, e))?;

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
        let content = std::fs::read_to_string(path).map_err(|e| format!("Failed to read '{}': {}", relative_path, e))?;
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

/// Tauri command: Parse a decision table from a markdown file.
#[tauri::command]
fn parse_decision_table(file_path: String) -> Result<DecisionTableInfo, String> {
  let content = std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read '{}': {}", file_path, e))?;

  let front_matter = parse_front_matter(&content).map_err(|e| format!("Failed to parse front matter: {}", e))?;
  let body = dsntk_type_registry::front_matter::extract_body(&content).unwrap_or("");

  let dt = dsntk_recognizer::from_markdown(body, false).map_err(|e| format!("Failed to parse decision table: {}", e))?;

  let input_columns: Vec<String> = dt.input_clauses.iter().map(|ic| ic.input_expression.clone()).collect();
  let output_columns: Vec<String> = dt.output_clauses.iter().map(|oc| oc.output_component_name.clone().unwrap_or_default()).collect();

  let rules: Vec<DecisionRuleInfo> = dt
    .rules
    .iter()
    .enumerate()
    .map(|(i, rule)| DecisionRuleInfo {
      index: i as u32,
      input_entries: rule.input_entries.iter().map(|ie| ie.text.clone()).collect(),
      output_entries: rule.output_entries.iter().map(|oe| oe.text.clone()).collect(),
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

/// Tauri command: Evaluate a FEEL expression with a JSON context.
#[tauri::command]
fn evaluate_feel_expression(expression: String, context_json: String) -> Result<String, String> {
  let scope = dsntk_feel::FeelScope::default();
  let node =
    dsntk_feel_parser::parse_expression(&scope, &expression, false).map_err(|e| format!("Failed to parse FEEL expression: {}", e))?;

  let input_value: serde_json::Value = serde_json::from_str(&context_json).map_err(|e| format!("Failed to parse context JSON: {}", e))?;
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
fn json_to_feel_value(value: &serde_json::Value) -> dsntk_feel::values::Value {
  use dsntk_feel::values::Value;
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

// ---------------------------------------------------------------------------
// Template metadata (mirrored from dsntk/src/templates/mod.rs)
// ---------------------------------------------------------------------------

/// Template metadata exposed to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateInfo {
  pub name: String,
  pub description: String,
  pub node_count: u32,
  pub features: Vec<String>,
}

/// Returns metadata for all built-in project templates.
fn built_in_templates() -> Vec<TemplateInfo> {
  vec![
    TemplateInfo {
      name: "loan-eligibility".to_string(),
      description: "Chained decisions with UNIQUE hit policy for loan approval".to_string(),
      node_count: 3,
      features: vec!["UNIQUE hit policy".into(), "Chained decisions".into(), "Risk rating".into()],
    },
    TemplateInfo {
      name: "insurance-pricing".to_string(),
      description: "BKM with age-based pricing, FEEL ranges, and literal expressions".to_string(),
      node_count: 3,
      features: vec!["BKM nodes".into(), "FEEL ranges".into(), "Literal expressions".into()],
    },
    TemplateInfo {
      name: "tax-calculator".to_string(),
      description: "Progressive tax brackets with numeric ranges and chained calculation".to_string(),
      node_count: 3,
      features: vec!["Numeric ranges".into(), "Progressive brackets".into(), "COLLECT policy".into()],
    },
    TemplateInfo {
      name: "order-routing".to_string(),
      description: "Multi-input decision tables for logistics branching".to_string(),
      node_count: 3,
      features: vec!["Multi-input tables".into(), "Logistics branching".into(), "FIRST policy".into()],
    },
    TemplateInfo {
      name: "compliance-checker".to_string(),
      description: "Knowledge sources, governed-by relationships, and boolean logic".to_string(),
      node_count: 4,
      features: vec!["Knowledge sources".into(), "Governed-by edges".into(), "Boolean logic".into()],
    },
    TemplateInfo {
      name: "scorecard".to_string(),
      description: "Weighted BKM scoring with chained decision contexts".to_string(),
      node_count: 5,
      features: vec!["BKM scoring".into(), "Weighted factors".into(), "Chained contexts".into()],
    },
  ]
}

/// Tauri command: List all available project templates.
#[tauri::command]
fn list_templates() -> Vec<TemplateInfo> {
  built_in_templates()
}

/// Tauri command: Create a new project from a built-in template.
///
/// Shells out to `dsntk new <template> <dest>` which handles all file scaffolding.
#[tauri::command]
fn create_project_from_template(template_name: String, dest_dir: String) -> Result<String, String> {
  let dest = Path::new(&dest_dir);
  if dest.exists() {
    return Err(format!("Directory '{}' already exists", dest_dir));
  }

  // Use the dsntk CLI to scaffold — it has the template files compiled in.
  let output = std::process::Command::new("dsntk")
    .args(["new", &template_name, &dest_dir])
    .output()
    .map_err(|e| format!("Failed to run 'dsntk new': {}. Is dsntk installed and on PATH?", e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("dsntk new failed: {}", stderr));
  }

  Ok(dest_dir)
}

// ---------------------------------------------------------------------------
// Workspace registry — persists recently opened projects
// ---------------------------------------------------------------------------

/// A recently opened project entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentProject {
  pub path: String,
  pub name: String,
  pub last_opened: String,
}

/// The on-disk workspace registry.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct WorkspaceRegistry {
  recent_projects: Vec<RecentProject>,
}

/// Returns the path to the workspace registry JSON file.
fn registry_path() -> PathBuf {
  let dir = dirs_next::data_local_dir().unwrap_or_else(|| PathBuf::from(".")).join("dsntk-explorer");
  let _ = std::fs::create_dir_all(&dir);
  dir.join("workspace.json")
}

/// Reads the workspace registry from disk.
fn read_registry() -> WorkspaceRegistry {
  let path = registry_path();
  if let Ok(contents) = std::fs::read_to_string(&path) {
    serde_json::from_str(&contents).unwrap_or_default()
  } else {
    WorkspaceRegistry::default()
  }
}

/// Writes the workspace registry to disk.
fn write_registry(registry: &WorkspaceRegistry) -> Result<(), String> {
  let path = registry_path();
  let json = serde_json::to_string_pretty(registry).map_err(|e| format!("Failed to serialize registry: {}", e))?;
  std::fs::write(&path, json).map_err(|e| format!("Failed to write registry: {}", e))
}

/// Tauri command: Get recently opened projects.
#[tauri::command]
fn get_recent_projects() -> Vec<RecentProject> {
  read_registry().recent_projects
}

/// Tauri command: Add or update a project in the recent projects list.
#[tauri::command]
fn add_recent_project(path: String, name: String) -> Result<(), String> {
  let mut reg = read_registry();
  let now = chrono::Utc::now().to_rfc3339();

  // Remove any existing entry with the same path.
  reg.recent_projects.retain(|p| p.path != path);

  // Insert at the front.
  reg.recent_projects.insert(
    0,
    RecentProject {
      path,
      name,
      last_opened: now,
    },
  );

  // Keep at most 20 recent projects.
  reg.recent_projects.truncate(20);

  write_registry(&reg)
}

/// Tauri command: Remove a project from recent projects.
#[tauri::command]
fn remove_recent_project(path: String) -> Result<(), String> {
  let mut reg = read_registry();
  reg.recent_projects.retain(|p| p.path != path);
  write_registry(&reg)
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![
      load_dmn_project,
      parse_decision_table,
      evaluate_feel_expression,
      list_templates,
      create_project_from_template,
      get_recent_projects,
      add_recent_project,
      remove_recent_project
    ])
    .run(tauri::generate_context!())
    .expect("error while running DSNTK Visual DMN Explorer");
}
