//! # DSNTK Visual DMN Explorer
//!
//! Tauri desktop application for visualizing and evaluating DMN models.

#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A node in the DMN flow graph for SvelteFlow visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowNode {
  pub id: String,
  pub label: String,
  pub node_type: String,
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
  pub model_name: String,
  pub model_namespace: String,
}

/// Trace result for a single node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeTrace {
  pub status: String,
  pub value: String,
}

/// Evaluation trace for the entire model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationTrace {
  pub node_results: HashMap<String, NodeTrace>,
  pub output_value: String,
}

/// Tauri command: Load a DMN model and return the flow graph.
#[tauri::command]
fn load_dmn_model(path: String) -> Result<FlowGraph, String> {
  let xml_content = std::fs::read_to_string(&path).map_err(|e| format!("Failed to read file '{}': {}", path, e))?;

  let definitions = dsntk_model::parse(&xml_content).map_err(|e| format!("Failed to parse DMN model: {}", e))?;

  let mut nodes = Vec::new();
  let mut edges = Vec::new();
  let mut edge_counter = 0u32;

  for drg_element in definitions.drg_elements() {
    match drg_element {
      dsntk_model::DrgElement::Decision(decision) => {
        use dsntk_model::{DmnElement, NamedElement};
        let id = decision.id().clone();
        let label = decision.feel_name().to_string();
        nodes.push(FlowNode {
          id: id.clone(),
          label,
          node_type: "decision".to_string(),
        });
        for req in decision.information_requirements() {
          if let Some(href) = req.required_decision() {
            edges.push(FlowEdge {
              id: format!("e{}", edge_counter),
              source: href.id().to_string(),
              target: id.clone(),
              edge_type: "information".to_string(),
            });
            edge_counter += 1;
          }
          if let Some(href) = req.required_input() {
            edges.push(FlowEdge {
              id: format!("e{}", edge_counter),
              source: href.id().to_string(),
              target: id.clone(),
              edge_type: "information".to_string(),
            });
            edge_counter += 1;
          }
        }
        for req in decision.knowledge_requirements() {
          edges.push(FlowEdge {
            id: format!("e{}", edge_counter),
            source: req.required_knowledge().id().to_string(),
            target: id.clone(),
            edge_type: "knowledge".to_string(),
          });
          edge_counter += 1;
        }
      }
      dsntk_model::DrgElement::InputData(input_data) => {
        use dsntk_model::{DmnElement, NamedElement};
        nodes.push(FlowNode {
          id: input_data.id().clone(),
          label: input_data.feel_name().to_string(),
          node_type: "inputData".to_string(),
        });
      }
      dsntk_model::DrgElement::BusinessKnowledgeModel(bkm) => {
        use dsntk_model::{DmnElement, NamedElement};
        let id = bkm.id().clone();
        nodes.push(FlowNode {
          id: id.clone(),
          label: bkm.feel_name().to_string(),
          node_type: "businessKnowledgeModel".to_string(),
        });
        for req in bkm.knowledge_requirements() {
          edges.push(FlowEdge {
            id: format!("e{}", edge_counter),
            source: req.required_knowledge().id().to_string(),
            target: id.clone(),
            edge_type: "knowledge".to_string(),
          });
          edge_counter += 1;
        }
      }
      dsntk_model::DrgElement::DecisionService(ds) => {
        use dsntk_model::{DmnElement, NamedElement};
        nodes.push(FlowNode {
          id: ds.id().clone(),
          label: ds.feel_name().to_string(),
          node_type: "decisionService".to_string(),
        });
      }
      dsntk_model::DrgElement::KnowledgeSource(ks) => {
        use dsntk_model::{DmnElement, NamedElement};
        nodes.push(FlowNode {
          id: ks.id().clone(),
          label: ks.feel_name().to_string(),
          node_type: "knowledgeSource".to_string(),
        });
      }
    }
  }

  use dsntk_model::{DmnElement, NamedElement};
  Ok(FlowGraph {
    nodes,
    edges,
    model_name: definitions.feel_name().to_string(),
    model_namespace: definitions.namespace().to_string(),
  })
}

/// Tauri command: Evaluate a DMN model with input data and return trace.
#[tauri::command]
fn evaluate_with_trace(model_path: String, input_json: String) -> Result<EvaluationTrace, String> {
  let xml_content = std::fs::read_to_string(&model_path).map_err(|e| format!("Failed to read file '{}': {}", model_path, e))?;

  let definitions = dsntk_model::parse(&xml_content).map_err(|e| format!("Failed to parse DMN model: {}", e))?;

  let model_evaluator =
    dsntk_model_evaluator::ModelEvaluator::new(&[definitions.clone()]).map_err(|e| format!("Failed to build model evaluator: {}", e))?;

  let input_value: serde_json::Value = serde_json::from_str(&input_json).map_err(|e| format!("Failed to parse input JSON: {}", e))?;

  let input_context = json_to_feel_context(&input_value);

  let mut node_results = HashMap::new();
  use dsntk_model::{DmnElement, NamedElement};
  let model_namespace = definitions.namespace().to_string();
  let model_name = definitions.feel_name().to_string();
  let mut final_output = String::from("null");

  for decision in definitions.decisions() {
    let decision_name = decision.feel_name().to_string();
    let result = model_evaluator.evaluate_invocable(&model_namespace, &model_name, &decision_name, &input_context);

    let status = if result.is_null() { "miss" } else { "hit" };
    let value_str = format!("{}", result);
    final_output = value_str.clone();

    node_results.insert(
      decision.id().clone(),
      NodeTrace {
        status: status.to_string(),
        value: value_str,
      },
    );
  }

  for input in definitions.input_data() {
    let input_name = input.feel_name().to_string();
    let provided = input_context.get_entry(&dsntk_feel::Name::from(input_name.as_str()));
    let (status, value_str) = if let Some(val) = provided {
      ("hit", format!("{}", val))
    } else {
      ("ignored", "not provided".to_string())
    };
    node_results.insert(
      input.id().clone(),
      NodeTrace {
        status: status.to_string(),
        value: value_str,
      },
    );
  }

  Ok(EvaluationTrace {
    node_results,
    output_value: final_output,
  })
}

/// Converts a JSON value to a FEEL context.
fn json_to_feel_context(value: &serde_json::Value) -> dsntk_feel::context::FeelContext {
  let mut ctx = dsntk_feel::context::FeelContext::default();
  if let serde_json::Value::Object(map) = value {
    for (key, val) in map {
      let name = dsntk_feel::Name::from(key.as_str());
      let feel_value = json_to_feel_value(val);
      ctx.set_entry(&name, feel_value);
    }
  }
  ctx
}

/// Converts a JSON value to a FEEL value.
fn json_to_feel_value(value: &serde_json::Value) -> dsntk_feel::values::Value {
  match value {
    serde_json::Value::Null => dsntk_feel::value_null!(),
    serde_json::Value::Bool(b) => dsntk_feel::values::Value::Boolean(*b),
    serde_json::Value::Number(n) => {
      if let Ok(num) = n.to_string().parse::<dsntk_feel::FeelNumber>() {
        dsntk_feel::values::Value::Number(num)
      } else {
        dsntk_feel::value_null!()
      }
    }
    serde_json::Value::String(s) => dsntk_feel::values::Value::String(s.clone()),
    serde_json::Value::Array(arr) => {
      let items: Vec<dsntk_feel::values::Value> = arr.iter().map(json_to_feel_value).collect();
      dsntk_feel::values::Value::List(items)
    }
    serde_json::Value::Object(map) => {
      let mut ctx = dsntk_feel::context::FeelContext::default();
      for (key, val) in map {
        let name = dsntk_feel::Name::from(key.as_str());
        ctx.set_entry(&name, json_to_feel_value(val));
      }
      dsntk_feel::values::Value::Context(ctx)
    }
  }
}

fn main() {
  tauri::Builder::default()
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_dialog::init())
    .invoke_handler(tauri::generate_handler![load_dmn_model, evaluate_with_trace])
    .run(tauri::generate_context!())
    .expect("error while running DSNTK Visual DMN Explorer");
}
