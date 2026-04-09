# DMN Evaluation Viewer — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build an interactive web UI that visualizes DMN decision evaluation as an animated DAG with full rule-level transparency.

**Architecture:** New `/api/v1/evaluate-trace` endpoint on dsntk-server returns a JSON evaluation trace (DAG structure + per-rule match data). A SvelteKit frontend in `viewer/` consumes this and renders the DAG using Svelte Flow with custom decision table nodes, animated edges, and step-through controls.

**Tech Stack:** Rust (Actix-web, serde), SvelteKit, TypeScript, @xyflow/svelte (Svelte Flow), dagre

**Spec:** `docs/superpowers/specs/2026-04-08-evaluation-viewer-design.md`

---

## File Structure

### Backend (Rust)

| File | Action | Purpose |
|---|---|---|
| `model-evaluator/src/trace.rs` | Create | Trace data structures: `EvaluationTrace`, `TraceStep`, `CellEvaluation` |
| `model-evaluator/src/lib.rs` | Modify | Add `pub mod trace;` |
| `model-evaluator/src/decision_table.rs` | Modify | Capture cell evaluations during rule matching |
| `model-evaluator/src/decision.rs` | Modify | Collect `TraceStep` after each decision evaluates |
| `model-evaluator/src/model_evaluator.rs` | Modify | Add `evaluate_invocable_traced()` method |
| `model-evaluator/src/model_definitions.rs` | Modify | Add `to_trace_graph()` method to extract DAG topology |
| `server/src/trace_handlers.rs` | Create | Actix-web handlers for `/api/v1/models` and `/api/v1/evaluate-trace` |
| `server/src/server.rs` | Modify | Register new routes |
| `server/src/lib.rs` | Modify | Add `mod trace_handlers;` |
| `server/Cargo.toml` | Modify | Add `actix-cors` dependency |
| `Cargo.toml` (root) | Modify | Add `actix-cors` to workspace dependencies |

### Frontend (SvelteKit)

| File | Action | Purpose |
|---|---|---|
| `viewer/package.json` | Create | SvelteKit project with dependencies |
| `viewer/svelte.config.js` | Create | SvelteKit config |
| `viewer/vite.config.ts` | Create | Vite config with API proxy |
| `viewer/tsconfig.json` | Create | TypeScript config |
| `viewer/src/app.html` | Create | HTML shell |
| `viewer/src/app.css` | Create | Global styles (dark theme) |
| `viewer/src/lib/types.ts` | Create | TypeScript types matching API contract |
| `viewer/src/lib/api.ts` | Create | API client functions |
| `viewer/src/lib/stores.ts` | Create | Svelte stores: traceStore, stepStore, inputStore |
| `viewer/src/lib/layout.ts` | Create | Dagre layout computation |
| `viewer/src/lib/components/InputPanel.svelte` | Create | Left panel: model selector + input fields |
| `viewer/src/lib/components/DagCanvas.svelte` | Create | Svelte Flow wrapper |
| `viewer/src/lib/components/StepControls.svelte` | Create | Playback toolbar |
| `viewer/src/lib/components/DecisionTableNode.svelte` | Create | Custom node: full table with row highlighting |
| `viewer/src/lib/components/InputDataNode.svelte` | Create | Custom node: input name + value |
| `viewer/src/lib/components/AnimatedEdge.svelte` | Create | Custom edge: pulse + value label |
| `viewer/src/routes/+page.svelte` | Create | Main page layout |

---

## Task 1: Trace Data Structures

**Files:**
- Create: `model-evaluator/src/trace.rs`
- Modify: `model-evaluator/src/lib.rs`
- Modify: `model-evaluator/Cargo.toml` (if serde not already a dep)

- [ ] **Step 1: Check if serde is already a dependency of model-evaluator**

Run: `grep serde model-evaluator/Cargo.toml`

- [ ] **Step 2: Create `model-evaluator/src/trace.rs` with trace structs**

```rust
//! Evaluation trace types for the DMN Evaluation Viewer.
//!
//! These types capture the full evaluation path through the DRG,
//! including per-rule match details and cell-level FEEL expression results.

use serde::Serialize;
use std::collections::HashMap;

/// Complete evaluation trace returned by the trace API.
#[derive(Debug, Clone, Serialize)]
pub struct EvaluationTrace {
  /// DAG structure: nodes and edges.
  pub graph: TraceGraph,
  /// Node IDs in the order they were evaluated (topological).
  pub evaluation_order: Vec<String>,
  /// Per-decision evaluation details.
  pub steps: Vec<TraceStep>,
}

/// DAG topology.
#[derive(Debug, Clone, Serialize)]
pub struct TraceGraph {
  pub nodes: Vec<TraceNode>,
  pub edges: Vec<TraceEdge>,
}

/// A node in the DAG — either an input or a decision table.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum TraceNode {
  #[serde(rename = "input_data")]
  InputData {
    id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<serde_json::Value>,
  },
  #[serde(rename = "decision_table")]
  DecisionTable {
    id: String,
    name: String,
    hit_policy: String,
    input_columns: Vec<String>,
    output_columns: Vec<String>,
    rules: Vec<TraceRule>,
  },
}

/// A rule (row) in a decision table.
#[derive(Debug, Clone, Serialize)]
pub struct TraceRule {
  pub index: usize,
  pub input_entries: Vec<String>,
  pub output_entries: Vec<String>,
}

/// An edge in the DAG.
#[derive(Debug, Clone, Serialize)]
pub struct TraceEdge {
  pub source: String,
  pub target: String,
  pub label: String,
}

/// Evaluation result for a single decision node.
#[derive(Debug, Clone, Serialize)]
pub struct TraceStep {
  pub node_id: String,
  pub input_values: HashMap<String, serde_json::Value>,
  pub matched_rules: Vec<usize>,
  pub output_value: serde_json::Value,
  pub cell_evaluations: Vec<CellEvaluation>,
}

/// Result of evaluating a single cell in the decision table.
/// Captured for all rules (not just matched) for a complete audit trail.
#[derive(Debug, Clone, Serialize)]
pub struct CellEvaluation {
  pub rule_index: usize,
  pub column_index: usize,
  pub expression: String,
  pub input_value: serde_json::Value,
  pub result: bool,
}

/// Collects trace data during evaluation.
/// Thread-local to avoid changing evaluator signatures.
#[derive(Debug, Default)]
pub struct TraceCollector {
  pub steps: Vec<TraceStep>,
}

impl TraceCollector {
  pub fn new() -> Self {
    Self { steps: Vec::new() }
  }

  pub fn push_step(&mut self, step: TraceStep) {
    self.steps.push(step);
  }
}

thread_local! {
  static TRACE_COLLECTOR: std::cell::RefCell<Option<TraceCollector>> = const { std::cell::RefCell::new(None) };
}

/// Enable trace collection for the current thread.
pub fn trace_start() {
  TRACE_COLLECTOR.with(|tc| {
    *tc.borrow_mut() = Some(TraceCollector::new());
  });
}

/// Disable trace collection and return collected data.
pub fn trace_finish() -> Option<TraceCollector> {
  TRACE_COLLECTOR.with(|tc| tc.borrow_mut().take())
}

/// Push a step if tracing is active.
pub fn trace_push_step(step: TraceStep) {
  TRACE_COLLECTOR.with(|tc| {
    if let Some(ref mut collector) = *tc.borrow_mut() {
      collector.push_step(step);
    }
  });
}

/// Returns true if tracing is currently active.
pub fn trace_is_active() -> bool {
  TRACE_COLLECTOR.with(|tc| tc.borrow().is_some())
}
```

- [ ] **Step 3: Add `pub mod trace;` to `model-evaluator/src/lib.rs`**

Find the existing module declarations in `model-evaluator/src/lib.rs` and add:

```rust
pub mod trace;
```

- [ ] **Step 4: Add serde_json dependency to model-evaluator if missing**

Check `model-evaluator/Cargo.toml`. If `serde_json` is not listed, add it under `[dependencies]`:

```toml
serde_json = { workspace = true }
```

Also ensure `serde` with `derive` feature is present:

```toml
serde = { workspace = true }
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo +stable build -p dsntk-model-evaluator`
Expected: Successful build with no errors.

- [ ] **Step 6: Commit**

```bash
git add model-evaluator/src/trace.rs model-evaluator/src/lib.rs model-evaluator/Cargo.toml
git commit -m "feat(model-evaluator): add evaluation trace data structures

Adds TraceNode, TraceStep, CellEvaluation types and a thread-local
TraceCollector for capturing evaluation paths through the DRG."
```

---

## Task 2: Graph Extraction from DefDefinitions

**Files:**
- Modify: `model-evaluator/src/model_definitions.rs:634-729`
- Test: inline unit test in `model_definitions.rs` or `trace.rs`

This task adds a method to extract the DAG topology (nodes + edges) from the existing `DefDefinitions` struct, producing `TraceGraph`.

- [ ] **Step 1: Write a test for graph extraction**

Add at the bottom of `model-evaluator/src/trace.rs`:

```rust
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_trace_graph_structure() {
    let graph = TraceGraph {
      nodes: vec![
        TraceNode::InputData {
          id: "input_age".to_string(),
          name: "Age".to_string(),
          value: Some(serde_json::json!(25)),
        },
        TraceNode::DecisionTable {
          id: "dt_eligibility".to_string(),
          name: "Eligibility".to_string(),
          hit_policy: "Unique".to_string(),
          input_columns: vec!["Age".to_string()],
          output_columns: vec!["Result".to_string()],
          rules: vec![TraceRule {
            index: 0,
            input_entries: vec![">=18".to_string()],
            output_entries: vec!["\"Eligible\"".to_string()],
          }],
        },
      ],
      edges: vec![TraceEdge {
        source: "input_age".to_string(),
        target: "dt_eligibility".to_string(),
        label: "Age".to_string(),
      }],
    };
    let json = serde_json::to_string(&graph).unwrap();
    assert!(json.contains("input_data"));
    assert!(json.contains("decision_table"));
    assert!(json.contains("input_age"));
  }

  #[test]
  fn test_trace_collector_thread_local() {
    trace_start();
    assert!(trace_is_active());
    trace_push_step(TraceStep {
      node_id: "test".to_string(),
      input_values: HashMap::new(),
      matched_rules: vec![0],
      output_value: serde_json::json!("ok"),
      cell_evaluations: vec![],
    });
    let collector = trace_finish().unwrap();
    assert_eq!(collector.steps.len(), 1);
    assert_eq!(collector.steps[0].node_id, "test");
    assert!(!trace_is_active());
  }
}
```

- [ ] **Step 2: Run tests to verify they pass**

Run: `cargo +stable test -p dsntk-model-evaluator trace::tests`
Expected: 2 tests pass.

- [ ] **Step 3: Add `to_trace_graph()` method to `DefDefinitions`**

In `model-evaluator/src/model_definitions.rs`, add a new method to the `impl DefDefinitions` block (after the existing methods around line 729):

```rust
  /// Extracts the DAG topology as a `TraceGraph` for the evaluation viewer.
  pub fn to_trace_graph(&self) -> crate::trace::TraceGraph {
    use crate::trace::{TraceEdge, TraceGraph, TraceNode, TraceRule};
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    // Add input data nodes.
    for (def_key, input_data) in &self.input_data {
      nodes.push(TraceNode::InputData {
        id: def_key.to_string(),
        name: input_data.name().to_string(),
        value: None,
      });
    }

    // Add decision nodes and edges.
    for (def_key, decision) in &self.decisions {
      // Extract decision table structure if present.
      let (hit_policy, input_columns, output_columns, rules) = if let Some(expression) = decision.decision_logic() {
        if let Some(decision_table) = expression.as_decision_table() {
          let hp = format!("{}", decision_table.hit_policy());
          let ic: Vec<String> = decision_table.input_clauses().map(|c| c.input_expression().to_string()).collect();
          let oc: Vec<String> = decision_table.output_clauses().map(|c| c.name().cloned().unwrap_or_default()).collect();
          let rs: Vec<TraceRule> = decision_table
            .rules()
            .enumerate()
            .map(|(i, rule)| TraceRule {
              index: i,
              input_entries: rule.input_entries().iter().map(|e| e.text().to_string()).collect(),
              output_entries: rule.output_entries().iter().map(|e| e.text().to_string()).collect(),
            })
            .collect();
          (hp, ic, oc, rs)
        } else {
          ("N/A".to_string(), vec![], vec![], vec![])
        }
      } else {
        ("N/A".to_string(), vec![], vec![], vec![])
      };

      nodes.push(TraceNode::DecisionTable {
        id: def_key.to_string(),
        name: decision.name().to_string(),
        hit_policy,
        input_columns,
        output_columns,
        rules,
      });

      // Add edges from information requirements.
      for req in decision.information_requirements() {
        if let Some(href) = req.required_decision() {
          let source_key = format!("{}#{}", href.namespace(), href.id());
          edges.push(TraceEdge {
            source: source_key,
            target: def_key.to_string(),
            label: decision.name().to_string(),
          });
        }
        if let Some(href) = req.required_input() {
          let source_key = format!("{}#{}", href.namespace(), href.id());
          edges.push(TraceEdge {
            source: source_key,
            target: def_key.to_string(),
            label: decision.name().to_string(),
          });
        }
      }
    }

    TraceGraph { nodes, edges }
  }
```

**Note:** This method references several accessors on `DefDecision` and the model types. You may need to verify that `decision.decision_logic()`, `expression.as_decision_table()`, `decision_table.input_clauses()`, etc. exist with these exact signatures. Check:
- `model-evaluator/src/model_definitions.rs` for `DefDecision` methods
- `model/src/model.rs` for `DecisionTable`, `InputClause`, `OutputClause`, `DecisionRule` accessors
- Adjust method names as needed to match the actual API (e.g. `input_expression()` on `InputClause`, `text()` on `InputEntry`/`OutputEntry`).

- [ ] **Step 4: Verify it compiles**

Run: `cargo +stable build -p dsntk-model-evaluator`
Expected: Successful build. Fix any accessor name mismatches.

- [ ] **Step 5: Commit**

```bash
git add model-evaluator/src/model_definitions.rs model-evaluator/src/trace.rs
git commit -m "feat(model-evaluator): extract DAG topology from DefDefinitions

Adds to_trace_graph() that converts model definitions into TraceGraph
(nodes + edges) for the evaluation viewer frontend."
```

---

## Task 3: Cell Evaluation Capture in Decision Table

**Files:**
- Modify: `model-evaluator/src/decision_table.rs:383-433`

This is the most delicate change — we instrument the existing rule evaluation loop to capture cell-level results when tracing is active.

- [ ] **Step 1: Add trace imports at the top of `decision_table.rs`**

Add after the existing imports:

```rust
use crate::trace::{trace_is_active, CellEvaluation};
```

- [ ] **Step 2: Modify `evaluate_parsed_decision_table` to capture cell evaluations**

In `decision_table.rs`, the function `evaluate_parsed_decision_table` (around line 383) has a loop that evaluates each rule's input entries. Modify it to collect `CellEvaluation` data when tracing is active.

Find the rule evaluation loop (approximately lines 399-420):

```rust
  let mut evaluated_rules = vec![];
  for parsed_rule in parsed_decision_table.rules.iter() {
    let mut input_entry_values = vec![];
    let mut matches = true;
    for evaluator in parsed_rule.input_entries_evaluators.iter() {
      let input_value: Value = evaluator(scope);
      let is_true = match &input_value {
        Value::ExpressionList(values) => values.iter().all(|value| value.is_true()),
        _ => input_value.is_true(),
      };
      if !is_true { matches = false; }
      input_entry_values.push(input_value);
    }
```

Replace with:

```rust
  let tracing = trace_is_active();
  let mut cell_evaluations: Vec<CellEvaluation> = Vec::new();
  let mut evaluated_rules = vec![];
  for (rule_index, parsed_rule) in parsed_decision_table.rules.iter().enumerate() {
    let mut input_entry_values = vec![];
    let mut matches = true;
    for (col_index, evaluator) in parsed_rule.input_entries_evaluators.iter().enumerate() {
      let input_value: Value = evaluator(scope);
      let is_true = match &input_value {
        Value::ExpressionList(values) => values.iter().all(|value| value.is_true()),
        _ => input_value.is_true(),
      };
      if tracing {
        cell_evaluations.push(CellEvaluation {
          rule_index,
          column_index: col_index,
          expression: String::new(), // filled by caller with original text
          input_value: serde_json::json!(format!("{}", input_value)),
          result: is_true,
        });
      }
      if !is_true { matches = false; }
      input_entry_values.push(input_value);
    }
```

- [ ] **Step 3: Return cell evaluations from the function**

The `EvaluatedDecisionTable` struct (around line 80) needs a new field. Add:

```rust
struct EvaluatedDecisionTable {
  component_names: Vec<Name>,
  output_values: Vec<Value>,
  default_output_values: Vec<Value>,
  evaluated_rules: Vec<EvaluatedRule>,
  cell_evaluations: Vec<CellEvaluation>,
}
```

Update the return statement of `evaluate_parsed_decision_table` to include `cell_evaluations`.

- [ ] **Step 4: Verify it compiles**

Run: `cargo +stable build -p dsntk-model-evaluator`
Expected: Build succeeds. You may need to update all places that construct `EvaluatedDecisionTable` to include the new field.

- [ ] **Step 5: Commit**

```bash
git add model-evaluator/src/decision_table.rs
git commit -m "feat(model-evaluator): capture cell-level evaluation results

When tracing is active, record per-cell match results (expression,
input value, boolean result) for every rule in the decision table."
```

---

## Task 4: TraceStep Collection in Decision Evaluator

**Files:**
- Modify: `model-evaluator/src/decision.rs:157-240`

- [ ] **Step 1: Add trace imports to `decision.rs`**

```rust
use crate::trace::{trace_is_active, trace_push_step, TraceStep};
```

- [ ] **Step 2: Add trace step collection after decision evaluation**

In the decision evaluator closure (around line 225-235 where the decision logic is executed and the result is placed into the context), add trace collection after the evaluation completes:

Find the section where the decision's output value is computed and added to the context. After that line, add:

```rust
      if trace_is_active() {
        let mut input_values_map = std::collections::HashMap::new();
        // Capture the input values that were used
        // (from the scope/context available at this point)
        let output_json = serde_json::to_value(format!("{}", &output_value)).unwrap_or(serde_json::Value::Null);
        trace_push_step(TraceStep {
          node_id: def_key.to_string(),
          input_values: input_values_map,
          matched_rules: vec![], // filled by decision_table.rs integration
          output_value: output_json,
          cell_evaluations: vec![], // filled by decision_table.rs integration
        });
      }
```

**Note:** The exact integration point depends on where `output_value` is available in the closure. The implementer should find where the decision logic evaluator is called (the `(evaluator)(scope)` call) and add the trace step immediately after. The `matched_rules` and `cell_evaluations` fields need coordination with the `decision_table.rs` changes from Task 3 — the decision table evaluator should store its per-evaluation results in the thread-local so this code can retrieve them.

- [ ] **Step 3: Verify it compiles**

Run: `cargo +stable build -p dsntk-model-evaluator`
Expected: Build succeeds.

- [ ] **Step 4: Run existing tests to verify no regression**

Run: `cargo +stable test -p dsntk-model-evaluator`
Expected: All existing tests pass. Tracing is not active during normal tests, so the `if trace_is_active()` guard means zero overhead.

- [ ] **Step 5: Commit**

```bash
git add model-evaluator/src/decision.rs
git commit -m "feat(model-evaluator): collect trace steps during decision evaluation

When tracing is active, push a TraceStep for each decision evaluated,
capturing the output value and evaluation order."
```

---

## Task 5: `evaluate_invocable_traced()` on ModelEvaluator

**Files:**
- Modify: `model-evaluator/src/model_evaluator.rs:56-129`

- [ ] **Step 1: Add the traced evaluation method**

Add after the existing `evaluate_invocable` method (around line 115):

```rust
  /// Evaluates an invocable with full trace collection.
  pub fn evaluate_invocable_traced(
    &self,
    model_namespace: &str,
    model_name: &str,
    invocable_name: &str,
    input_data: &FeelContext,
  ) -> (Value, Option<crate::trace::EvaluationTrace>) {
    use crate::trace::{trace_finish, trace_start};

    // Start trace collection.
    trace_start();

    // Run normal evaluation (trace data collected via thread-local).
    let result = self.evaluate_invocable(model_namespace, model_name, invocable_name, input_data);

    // Finish collection and build trace.
    let trace = trace_finish().map(|collector| {
      let graph = self.model_definitions().to_trace_graph();
      let evaluation_order: Vec<String> = collector.steps.iter().map(|s| s.node_id.clone()).collect();
      crate::trace::EvaluationTrace {
        graph,
        evaluation_order,
        steps: collector.steps,
      }
    });

    (result, trace)
  }
```

- [ ] **Step 2: Expose `model_definitions()` accessor if not already public**

Check if `ModelEvaluator` has a public accessor for its `DefDefinitions`. If not, add one:

```rust
  pub fn model_definitions(&self) -> &DefDefinitions {
    &self.model_definitions
  }
```

The `ModelEvaluator` struct fields are defined around line 30. Check what fields exist and how `DefDefinitions` is stored (it may be accessed through the `ModelBuilder` or stored directly).

- [ ] **Step 3: Verify it compiles**

Run: `cargo +stable build -p dsntk-model-evaluator`
Expected: Build succeeds.

- [ ] **Step 4: Commit**

```bash
git add model-evaluator/src/model_evaluator.rs
git commit -m "feat(model-evaluator): add evaluate_invocable_traced() method

Wraps normal evaluation with trace collection, returning both the
result Value and the full EvaluationTrace for the viewer."
```

---

## Task 6: Server Trace Handlers

**Files:**
- Create: `server/src/trace_handlers.rs`
- Modify: `server/src/server.rs:48-71`
- Modify: `server/src/lib.rs`
- Modify: `server/Cargo.toml`
- Modify: `Cargo.toml` (root)

- [ ] **Step 1: Add `actix-cors` to workspace dependencies**

In root `Cargo.toml`, add under `[workspace.dependencies]`:

```toml
actix-cors = "0.7.0"
```

In `server/Cargo.toml`, add under `[dependencies]`:

```toml
actix-cors = { workspace = true }
```

Also add `dsntk-model-evaluator` if not already present (needed for trace types):

```toml
dsntk-model-evaluator = { workspace = true }
```

- [ ] **Step 2: Create `server/src/trace_handlers.rs`**

```rust
//! Handlers for the evaluation trace API (`/api/v1/`).

use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::data::ApplicationData;

/// Request body for `/api/v1/evaluate-trace`.
#[derive(Deserialize)]
struct EvaluateTraceRequest {
  namespace: String,
  invocable: String,
  inputs: HashMap<String, serde_json::Value>,
}

/// Response for `/api/v1/models`.
#[derive(Serialize)]
struct ModelsResponse {
  models: Vec<ModelInfo>,
}

#[derive(Serialize)]
struct ModelInfo {
  namespace: String,
  name: String,
  invocables: Vec<String>,
  inputs: Vec<InputInfo>,
}

#[derive(Serialize)]
struct InputInfo {
  name: String,
  feel_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  allowed_values: Option<Vec<String>>,
  optional: bool,
}

/// `GET /api/v1/models` — returns available models and their input metadata.
#[get("/api/v1/models")]
async fn get_models(data: web::Data<ApplicationData>) -> HttpResponse {
  // TODO: Iterate workspaces and extract model metadata.
  // This requires access to the model definitions and type registry
  // through the workspace's evaluators.
  //
  // For now, return an empty list — the implementer should wire this
  // to `data.workspaces` once the accessor pattern is clear.
  let response = ModelsResponse { models: vec![] };
  HttpResponse::Ok().json(response)
}

/// `POST /api/v1/evaluate-trace` — evaluates with full trace.
#[post("/api/v1/evaluate-trace")]
async fn evaluate_trace(
  body: web::Json<EvaluateTraceRequest>,
  data: web::Data<ApplicationData>,
) -> HttpResponse {
  // Convert JSON inputs to FeelContext.
  let mut input_ctx = dsntk_feel::context::FeelContext::default();
  for (key, value) in &body.inputs {
    let feel_value = json_to_feel_value(value);
    input_ctx.set_entry(&dsntk_feel::Name::from(key.as_str()), feel_value);
  }

  // Look up the invocable and evaluate with tracing.
  // The invocable path format used by workspaces needs to be constructed
  // from namespace + invocable name.
  let invocable_path = format!("{}/{}", body.namespace, body.invocable);

  // Access the workspace's evaluator for traced evaluation.
  if let Some((workspace_name, model_namespace, model_name, invocable_name)) =
    data.workspaces.invocables.get(&invocable_path)
  {
    if let Some(evaluator) = data.workspaces.evaluators.get(workspace_name) {
      let (result, trace) =
        evaluator.evaluate_invocable_traced(model_namespace, model_name, invocable_name, &input_ctx);

      #[derive(Serialize)]
      struct TraceResponse {
        result: serde_json::Value,
        #[serde(skip_serializing_if = "Option::is_none")]
        trace: Option<dsntk_model_evaluator::trace::EvaluationTrace>,
      }

      let response = TraceResponse {
        result: serde_json::json!(format!("{}", result)),
        trace,
      };
      return HttpResponse::Ok().json(response);
    }
  }

  HttpResponse::NotFound().json(serde_json::json!({
    "error": format!("invocable not found: {}", invocable_path)
  }))
}

/// Convert a JSON value to a FEEL value (basic types).
fn json_to_feel_value(value: &serde_json::Value) -> dsntk_feel::values::Value {
  match value {
    serde_json::Value::Number(n) => {
      if let Some(f) = n.as_f64() {
        dsntk_feel::values::Value::Number(dsntk_feel_number::FeelNumber::from(f))
      } else {
        dsntk_feel::values::Value::Null(Some("invalid number".to_string()))
      }
    }
    serde_json::Value::String(s) => dsntk_feel::values::Value::String(s.clone()),
    serde_json::Value::Bool(b) => dsntk_feel::values::Value::Boolean(*b),
    serde_json::Value::Null => dsntk_feel::values::Value::Null(None),
    _ => dsntk_feel::values::Value::Null(Some("unsupported JSON type".to_string())),
  }
}

/// Register trace API routes.
pub fn config(cfg: &mut web::ServiceConfig) {
  cfg.service(get_models);
  cfg.service(evaluate_trace);
}
```

- [ ] **Step 3: Add module and register routes in `server/src/lib.rs`**

Add:

```rust
mod trace_handlers;
```

- [ ] **Step 4: Register trace routes in `server/src/server.rs`**

In the `App::new()` builder (around line 65-71), add the trace config and CORS:

```rust
use actix_cors::Cors;

App::new()
  .wrap(
    Cors::default()
      .allow_any_origin()
      .allow_any_method()
      .allow_any_header()
      .max_age(3600),
  )
  .app_data(application_data.clone())
  .app_data(web::PayloadConfig::new(4 * 1024 * 1024))
  .configure(config)
  .configure(crate::trace_handlers::config)
  .default_service(web::route().to(not_found))
```

- [ ] **Step 5: Verify it compiles**

Run: `cargo +stable build -p dsntk-server`
Expected: Build succeeds. You may need to fix import paths for FEEL types — check the actual public API of `dsntk-feel` and `dsntk-feel-number`.

- [ ] **Step 6: Run existing server tests**

Run: `cargo +stable test -p dsntk-server`
Expected: All existing tests pass.

- [ ] **Step 7: Commit**

```bash
git add server/src/trace_handlers.rs server/src/server.rs server/src/lib.rs server/Cargo.toml Cargo.toml
git commit -m "feat(server): add /api/v1/evaluate-trace and /api/v1/models endpoints

New REST endpoints for the evaluation viewer. evaluate-trace returns
the full DAG structure and per-rule evaluation trace. CORS enabled
for local development."
```

---

## Task 7: SvelteKit Project Scaffold

**Files:**
- Create: `viewer/` directory with SvelteKit project files

- [ ] **Step 1: Scaffold SvelteKit project**

Run from the repo root:

```bash
cd viewer && npx sv create . --template minimal --types ts --no-add-ons --no-install
```

If `sv` (the SvelteKit CLI) is not available, create the project manually. Either way, the result should be a working SvelteKit project in `viewer/`.

- [ ] **Step 2: Install dependencies**

```bash
cd viewer && npm install
npm install @xyflow/svelte dagre @types/dagre
```

- [ ] **Step 3: Configure Vite proxy for the dsntk API**

Create/modify `viewer/vite.config.ts`:

```typescript
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
});
```

- [ ] **Step 4: Verify dev server starts**

```bash
cd viewer && npm run dev
```

Expected: SvelteKit dev server starts on port 5173.

- [ ] **Step 5: Commit**

```bash
git add viewer/
git commit -m "feat(viewer): scaffold SvelteKit project

Minimal SvelteKit app with Svelte Flow, dagre, and API proxy
configured for the dsntk server."
```

---

## Task 8: TypeScript Types and API Client

**Files:**
- Create: `viewer/src/lib/types.ts`
- Create: `viewer/src/lib/api.ts`

- [ ] **Step 1: Create TypeScript types matching the API contract**

Create `viewer/src/lib/types.ts`:

```typescript
/** A node in the evaluation DAG. */
export type TraceNode =
  | {
      type: 'input_data';
      id: string;
      name: string;
      value?: unknown;
    }
  | {
      type: 'decision_table';
      id: string;
      name: string;
      hit_policy: string;
      input_columns: string[];
      output_columns: string[];
      rules: TraceRule[];
    };

export interface TraceRule {
  index: number;
  input_entries: string[];
  output_entries: string[];
}

export interface TraceEdge {
  source: string;
  target: string;
  label: string;
}

export interface TraceGraph {
  nodes: TraceNode[];
  edges: TraceEdge[];
}

export interface CellEvaluation {
  rule_index: number;
  column_index: number;
  expression: string;
  input_value: unknown;
  result: boolean;
}

export interface TraceStep {
  node_id: string;
  input_values: Record<string, unknown>;
  matched_rules: number[];
  output_value: unknown;
  cell_evaluations: CellEvaluation[];
}

export interface EvaluationTrace {
  graph: TraceGraph;
  evaluation_order: string[];
  steps: TraceStep[];
}

export interface TraceResponse {
  result: unknown;
  trace?: EvaluationTrace;
}

export interface InputInfo {
  name: string;
  feel_type: string;
  allowed_values?: string[];
  optional: boolean;
}

export interface ModelInfo {
  namespace: string;
  name: string;
  invocables: string[];
  inputs: InputInfo[];
}

export interface ModelsResponse {
  models: ModelInfo[];
}

/** Visual state of a node during step-through. */
export type NodeEvalState = 'unevaluated' | 'evaluating' | 'evaluated';

/** Visual state of an edge during step-through. */
export type EdgeEvalState = 'inactive' | 'animating' | 'completed';
```

- [ ] **Step 2: Create API client**

Create `viewer/src/lib/api.ts`:

```typescript
import type { ModelsResponse, TraceResponse } from './types';

const BASE_URL = '/api/v1';

export async function fetchModels(): Promise<ModelsResponse> {
  const res = await fetch(`${BASE_URL}/models`);
  if (!res.ok) throw new Error(`Failed to fetch models: ${res.statusText}`);
  return res.json();
}

export async function evaluateTrace(
  namespace: string,
  invocable: string,
  inputs: Record<string, unknown>
): Promise<TraceResponse> {
  const res = await fetch(`${BASE_URL}/evaluate-trace`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ namespace, invocable, inputs }),
  });
  if (!res.ok) throw new Error(`Evaluation failed: ${res.statusText}`);
  return res.json();
}
```

- [ ] **Step 3: Commit**

```bash
git add viewer/src/lib/types.ts viewer/src/lib/api.ts
git commit -m "feat(viewer): add TypeScript types and API client

Types match the /api/v1/evaluate-trace response contract.
API client handles model listing and traced evaluation."
```

---

## Task 9: Svelte Stores

**Files:**
- Create: `viewer/src/lib/stores.ts`

- [ ] **Step 1: Create stores**

Create `viewer/src/lib/stores.ts`:

```typescript
import { writable, derived } from 'svelte/store';
import type { EvaluationTrace, ModelInfo, TraceStep } from './types';

/** Currently selected model. */
export const selectedModel = writable<ModelInfo | null>(null);

/** Current input values. */
export const inputValues = writable<Record<string, unknown>>({});

/** Full evaluation trace from the last API call. */
export const traceData = writable<EvaluationTrace | null>(null);

/** Current step index for the animated step-through (0 = nothing evaluated yet). */
export const currentStep = writable<number>(0);

/** Total number of steps. */
export const totalSteps = derived(traceData, ($trace) =>
  $trace ? $trace.steps.length : 0
);

/** Steps that should be visible at the current step index. */
export const visibleSteps = derived(
  [traceData, currentStep],
  ([$trace, $step]) => {
    if (!$trace) return [];
    return $trace.steps.slice(0, $step);
  }
);

/** Set of node IDs that have been fully evaluated at the current step. */
export const evaluatedNodeIds = derived(visibleSteps, ($steps) =>
  new Set($steps.map((s: TraceStep) => s.node_id))
);

/** The node ID currently being evaluated (the step we're on). */
export const evaluatingNodeId = derived(
  [traceData, currentStep],
  ([$trace, $step]) => {
    if (!$trace || $step <= 0 || $step > $trace.steps.length) return null;
    return $trace.steps[$step - 1]?.node_id ?? null;
  }
);

/** Map from node ID to its TraceStep (for evaluated nodes). */
export const stepByNodeId = derived(visibleSteps, ($steps) => {
  const map = new Map<string, TraceStep>();
  for (const step of $steps) {
    map.set(step.node_id, step);
  }
  return map;
});
```

- [ ] **Step 2: Commit**

```bash
git add viewer/src/lib/stores.ts
git commit -m "feat(viewer): add Svelte stores for trace state management

Stores for selected model, input values, evaluation trace, and
step-through animation state with derived computed values."
```

---

## Task 10: Dagre Layout Utility

**Files:**
- Create: `viewer/src/lib/layout.ts`

- [ ] **Step 1: Create layout utility**

Create `viewer/src/lib/layout.ts`:

```typescript
import dagre from 'dagre';
import type { TraceGraph } from './types';
import type { Node, Edge } from '@xyflow/svelte';

const NODE_WIDTH = 320;
const NODE_HEIGHT_INPUT = 60;
const NODE_HEIGHT_DECISION = 200;

export function computeLayout(graph: TraceGraph): { nodes: Node[]; edges: Edge[] } {
  const g = new dagre.graphlib.Graph();
  g.setDefaultEdgeLabel(() => ({}));
  g.setGraph({ rankdir: 'TB', nodesep: 60, ranksep: 80 });

  // Add nodes to dagre.
  for (const node of graph.nodes) {
    const height = node.type === 'input_data' ? NODE_HEIGHT_INPUT : NODE_HEIGHT_DECISION;
    g.setNode(node.id, { width: NODE_WIDTH, height });
  }

  // Add edges to dagre.
  for (const edge of graph.edges) {
    g.setEdge(edge.source, edge.target);
  }

  dagre.layout(g);

  // Convert to Svelte Flow format.
  const nodes: Node[] = graph.nodes.map((node) => {
    const pos = g.node(node.id);
    const height = node.type === 'input_data' ? NODE_HEIGHT_INPUT : NODE_HEIGHT_DECISION;
    return {
      id: node.id,
      type: node.type === 'input_data' ? 'inputData' : 'decisionTable',
      position: { x: pos.x - NODE_WIDTH / 2, y: pos.y - height / 2 },
      data: node,
    };
  });

  const edges: Edge[] = graph.edges.map((edge, i) => ({
    id: `e-${i}`,
    source: edge.source,
    target: edge.target,
    type: 'animated',
    data: { label: edge.label },
  }));

  return { nodes, edges };
}
```

- [ ] **Step 2: Commit**

```bash
git add viewer/src/lib/layout.ts
git commit -m "feat(viewer): add dagre layout computation

Converts TraceGraph into positioned Svelte Flow nodes and edges
using top-to-bottom dagre layout."
```

---

## Task 11: Custom Svelte Flow Nodes

**Files:**
- Create: `viewer/src/lib/components/InputDataNode.svelte`
- Create: `viewer/src/lib/components/DecisionTableNode.svelte`

- [ ] **Step 1: Create InputDataNode**

Create `viewer/src/lib/components/InputDataNode.svelte`:

```svelte
<script lang="ts">
  import { Handle, Position } from '@xyflow/svelte';
  import { evaluatedNodeIds } from '$lib/stores';

  export let id: string;
  export let data: { name: string; value?: unknown };

  $: isEvaluated = $evaluatedNodeIds.has(id);
</script>

<div class="input-node" class:evaluated={isEvaluated}>
  <Handle type="source" position={Position.Bottom} />
  <div class="name">{data.name}</div>
  {#if data.value !== undefined && data.value !== null}
    <div class="value">{data.value}</div>
  {/if}
</div>

<style>
  .input-node {
    background: #0d1117;
    border: 2px solid #58a6ff;
    border-radius: 12px;
    padding: 8px 16px;
    font-family: monospace;
    min-width: 120px;
    text-align: center;
  }
  .input-node.evaluated {
    border-color: #3fb950;
    background: #0d1117;
  }
  .name {
    color: #c9d1d9;
    font-size: 12px;
  }
  .value {
    color: #58a6ff;
    font-size: 14px;
    font-weight: bold;
    margin-top: 2px;
  }
  .evaluated .value {
    color: #3fb950;
  }
</style>
```

- [ ] **Step 2: Create DecisionTableNode**

Create `viewer/src/lib/components/DecisionTableNode.svelte`:

```svelte
<script lang="ts">
  import { Handle, Position } from '@xyflow/svelte';
  import { evaluatedNodeIds, evaluatingNodeId, stepByNodeId } from '$lib/stores';
  import type { TraceRule } from '$lib/types';

  export let id: string;
  export let data: {
    name: string;
    hit_policy: string;
    input_columns: string[];
    output_columns: string[];
    rules: TraceRule[];
  };

  $: isEvaluated = $evaluatedNodeIds.has(id);
  $: isEvaluating = $evaluatingNodeId === id;
  $: step = $stepByNodeId.get(id);
  $: matchedRules = step ? new Set(step.matched_rules) : new Set<number>();
</script>

<div
  class="dt-node"
  class:evaluated={isEvaluated}
  class:evaluating={isEvaluating}
>
  <Handle type="target" position={Position.Top} />
  <Handle type="source" position={Position.Bottom} />

  <!-- Header -->
  <div class="header">
    <span class="name">{data.name}</span>
    <span class="hit-policy">{data.hit_policy}</span>
  </div>

  <!-- Column headers -->
  <div class="columns">
    {#each data.input_columns as col}
      <span class="col input-col">{col}</span>
    {/each}
    <span class="col-sep">→</span>
    {#each data.output_columns as col}
      <span class="col output-col">{col}</span>
    {/each}
  </div>

  <!-- Rules -->
  <div class="rules">
    {#each data.rules as rule}
      <div class="rule" class:matched={matchedRules.has(rule.index)}>
        {#each rule.input_entries as entry}
          <span class="cell">{entry}</span>
        {/each}
        <span class="cell-sep">→</span>
        {#each rule.output_entries as entry}
          <span class="cell output-cell">{entry}</span>
        {/each}
      </div>
    {/each}
  </div>

  <!-- Result footer -->
  {#if step}
    <div class="footer">
      <span>Rule {step.matched_rules.join(', ')} matched</span>
      <span class="result">→ {step.output_value}</span>
    </div>
  {/if}
</div>

<style>
  .dt-node {
    background: #0d1117;
    border: 2px dashed #30363d;
    border-radius: 8px;
    font-family: monospace;
    font-size: 11px;
    min-width: 280px;
    overflow: hidden;
  }
  .dt-node.evaluating {
    border: 2px solid #d29922;
    box-shadow: 0 0 12px rgba(210, 153, 34, 0.3);
  }
  .dt-node.evaluated {
    border: 2px solid #3fb950;
  }

  .header {
    background: #161b22;
    padding: 8px 12px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #30363d;
  }
  .evaluated .header {
    background: #1a3a1a;
  }
  .name {
    color: #c9d1d9;
    font-weight: bold;
    font-size: 12px;
  }
  .evaluated .name {
    color: #3fb950;
  }
  .hit-policy {
    background: #21262d;
    color: #8b949e;
    padding: 2px 6px;
    border-radius: 4px;
    font-size: 9px;
  }

  .columns {
    display: flex;
    gap: 8px;
    padding: 4px 12px;
    background: #161b22;
    border-bottom: 1px solid #30363d;
    color: #8b949e;
    font-size: 10px;
  }
  .col-sep, .cell-sep {
    color: #484f58;
  }

  .rules {
    padding: 2px 0;
  }
  .rule {
    display: flex;
    gap: 8px;
    padding: 4px 12px;
    color: #484f58;
    border-bottom: 1px solid #21262d;
  }
  .rule.matched {
    background: #1a3a1a;
    border-left: 3px solid #3fb950;
    color: #3fb950;
  }
  .cell {
    flex: 1;
  }
  .output-cell {
    font-weight: bold;
  }

  .footer {
    background: #1a3a1a;
    padding: 6px 12px;
    border-top: 1px solid #3fb950;
    color: #3fb950;
    display: flex;
    justify-content: space-between;
    font-size: 10px;
  }
  .result {
    font-weight: bold;
  }
</style>
```

- [ ] **Step 3: Commit**

```bash
git add viewer/src/lib/components/InputDataNode.svelte viewer/src/lib/components/DecisionTableNode.svelte
git commit -m "feat(viewer): add custom Svelte Flow node components

InputDataNode shows input name + value with blue/green theming.
DecisionTableNode renders full table with matched-row highlighting."
```

---

## Task 12: Animated Edge Component

**Files:**
- Create: `viewer/src/lib/components/AnimatedEdge.svelte`

- [ ] **Step 1: Create AnimatedEdge**

Create `viewer/src/lib/components/AnimatedEdge.svelte`:

```svelte
<script lang="ts">
  import { BaseEdge, getBezierPath } from '@xyflow/svelte';
  import { evaluatedNodeIds, evaluatingNodeId } from '$lib/stores';

  export let id: string;
  export let sourceX: number;
  export let sourceY: number;
  export let targetX: number;
  export let targetY: number;
  export let sourcePosition: string;
  export let targetPosition: string;
  export let data: { label: string } | undefined;

  $: [path, labelX, labelY] = getBezierPath({
    sourceX,
    sourceY,
    targetX,
    targetY,
    sourcePosition,
    targetPosition,
  });

  // Determine edge state based on target node.
  // Extract target from edge ID pattern or use store lookup.
  $: targetId = id.split('-').slice(1).join('-'); // rough — real impl uses edge data
  $: isCompleted = $evaluatedNodeIds.has(targetId);
  $: isAnimating = $evaluatingNodeId === targetId;
</script>

<BaseEdge
  {id}
  {path}
  style={isCompleted
    ? 'stroke: #3fb950; stroke-width: 2;'
    : isAnimating
      ? 'stroke: #d29922; stroke-width: 2;'
      : 'stroke: #30363d; stroke-width: 1.5; stroke-dasharray: 5 5;'}
/>

{#if isAnimating}
  <circle r="4" fill="#d29922">
    <animateMotion dur="0.8s" repeatCount="indefinite" path={path} />
  </circle>
{/if}

{#if isCompleted && data?.label}
  <foreignObject
    x={labelX - 40}
    y={labelY - 10}
    width="80"
    height="20"
    class="edge-label-container"
  >
    <div class="edge-label">→ {data.label}</div>
  </foreignObject>
{/if}

<style>
  .edge-label {
    background: #1a3a1a;
    color: #3fb950;
    font-size: 9px;
    font-family: monospace;
    padding: 2px 6px;
    border-radius: 4px;
    text-align: center;
    white-space: nowrap;
  }
</style>
```

**Note:** The `<animateMotion>` SVG element creates the pulse effect along the edge path. The `BaseEdge` component from Svelte Flow renders the path itself. The implementer should verify the exact Svelte Flow API for custom edges — the props may differ slightly between `@xyflow/svelte` versions.

- [ ] **Step 2: Commit**

```bash
git add viewer/src/lib/components/AnimatedEdge.svelte
git commit -m "feat(viewer): add animated edge component

Custom Svelte Flow edge with pulse animation during evaluation
and value labels on completed edges."
```

---

## Task 13: Step Controls Component

**Files:**
- Create: `viewer/src/lib/components/StepControls.svelte`

- [ ] **Step 1: Create StepControls**

Create `viewer/src/lib/components/StepControls.svelte`:

```svelte
<script lang="ts">
  import { currentStep, totalSteps } from '$lib/stores';

  let playing = false;
  let speed = 1000; // ms per step
  let interval: ReturnType<typeof setInterval> | null = null;

  function stepForward() {
    currentStep.update((n) => Math.min(n + 1, $totalSteps));
  }

  function stepBack() {
    currentStep.update((n) => Math.max(n - 1, 0));
  }

  function skipToStart() {
    currentStep.set(0);
  }

  function skipToEnd() {
    currentStep.set($totalSteps);
  }

  function togglePlay() {
    if (playing) {
      stopPlaying();
    } else {
      startPlaying();
    }
  }

  function startPlaying() {
    playing = true;
    interval = setInterval(() => {
      currentStep.update((n) => {
        if (n >= $totalSteps) {
          stopPlaying();
          return n;
        }
        return n + 1;
      });
    }, speed);
  }

  function stopPlaying() {
    playing = false;
    if (interval) {
      clearInterval(interval);
      interval = null;
    }
  }

  // Clean up on destroy.
  import { onDestroy } from 'svelte';
  onDestroy(() => {
    if (interval) clearInterval(interval);
  });
</script>

<div class="controls">
  <button on:click={skipToStart} title="Skip to start">⏮</button>
  <button on:click={stepBack} title="Step back">◀</button>
  <button on:click={togglePlay} class:playing title={playing ? 'Pause' : 'Play'}>
    {playing ? '⏸' : '▶'}
  </button>
  <button on:click={stepForward} title="Step forward">▶</button>
  <button on:click={skipToEnd} title="Skip to end">⏭</button>
  <span class="counter">Step {$currentStep} / {$totalSteps}</span>
  <label class="speed">
    <input type="range" min="200" max="3000" step="100" bind:value={speed} />
    {speed}ms
  </label>
</div>

<style>
  .controls {
    display: flex;
    align-items: center;
    gap: 4px;
    background: #161b22;
    border: 1px solid #30363d;
    border-radius: 8px;
    padding: 6px 10px;
  }
  button {
    background: #21262d;
    border: 1px solid #30363d;
    border-radius: 6px;
    color: #c9d1d9;
    padding: 6px 10px;
    cursor: pointer;
    font-size: 12px;
  }
  button:hover {
    background: #30363d;
  }
  button.playing {
    background: #238636;
    border-color: #2ea043;
    color: #fff;
  }
  .counter {
    color: #8b949e;
    font-size: 12px;
    margin-left: 8px;
    font-family: monospace;
  }
  .speed {
    color: #8b949e;
    font-size: 10px;
    margin-left: 8px;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .speed input {
    width: 80px;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add viewer/src/lib/components/StepControls.svelte
git commit -m "feat(viewer): add step-through playback controls

Play/pause, step forward/back, skip, speed slider, and step counter
for animated evaluation walkthrough."
```

---

## Task 14: Input Panel Component

**Files:**
- Create: `viewer/src/lib/components/InputPanel.svelte`

- [ ] **Step 1: Create InputPanel**

Create `viewer/src/lib/components/InputPanel.svelte`:

```svelte
<script lang="ts">
  import { selectedModel, inputValues, traceData, currentStep } from '$lib/stores';
  import { fetchModels, evaluateTrace } from '$lib/api';
  import type { ModelInfo } from '$lib/types';
  import { onMount } from 'svelte';

  let models: ModelInfo[] = [];
  let loading = false;
  let error = '';

  onMount(async () => {
    try {
      const res = await fetchModels();
      models = res.models;
      if (models.length > 0) {
        selectedModel.set(models[0]);
        initInputs(models[0]);
      }
    } catch (e) {
      error = `Failed to load models: ${e}`;
    }
  });

  function initInputs(model: ModelInfo) {
    const defaults: Record<string, unknown> = {};
    for (const input of model.inputs) {
      switch (input.feel_type) {
        case 'number':
          defaults[input.name] = 0;
          break;
        case 'boolean':
          defaults[input.name] = false;
          break;
        default:
          defaults[input.name] = input.allowed_values?.[0] ?? '';
      }
    }
    inputValues.set(defaults);
  }

  function onModelChange(event: Event) {
    const target = event.target as HTMLSelectElement;
    const model = models.find((m) => m.namespace === target.value);
    if (model) {
      selectedModel.set(model);
      initInputs(model);
    }
  }

  function onInputChange(name: string, value: unknown) {
    inputValues.update((v) => ({ ...v, [name]: value }));
  }

  async function evaluate() {
    if (!$selectedModel) return;
    loading = true;
    error = '';
    try {
      const res = await evaluateTrace(
        $selectedModel.namespace,
        $selectedModel.invocables[0],
        $inputValues
      );
      traceData.set(res.trace ?? null);
      currentStep.set(0);
    } catch (e) {
      error = `Evaluation failed: ${e}`;
    } finally {
      loading = false;
    }
  }
</script>

<div class="panel">
  <!-- Model Selector -->
  <div class="section">
    <label class="label">Model</label>
    <select class="select" on:change={onModelChange}>
      {#each models as model}
        <option value={model.namespace}>{model.name}</option>
      {/each}
    </select>
  </div>

  <!-- Input Fields -->
  {#if $selectedModel}
    <div class="section">
      <label class="label">Input Data</label>
      {#each $selectedModel.inputs as input}
        <div class="field">
          <label class="field-label">{input.name}</label>
          {#if input.allowed_values}
            <select
              class="select"
              value={$inputValues[input.name]}
              on:change={(e) => onInputChange(input.name, (e.target as HTMLSelectElement).value)}
            >
              {#each input.allowed_values as val}
                <option value={val}>{val}</option>
              {/each}
            </select>
          {:else if input.feel_type === 'number'}
            <input
              class="input"
              type="number"
              value={$inputValues[input.name]}
              on:input={(e) => onInputChange(input.name, Number((e.target as HTMLInputElement).value))}
            />
          {:else if input.feel_type === 'boolean'}
            <label class="toggle">
              <input
                type="checkbox"
                checked={$inputValues[input.name] === true}
                on:change={(e) => onInputChange(input.name, (e.target as HTMLInputElement).checked)}
              />
              {$inputValues[input.name] ? 'true' : 'false'}
            </label>
          {:else}
            <input
              class="input"
              type="text"
              value={$inputValues[input.name]}
              on:input={(e) => onInputChange(input.name, (e.target as HTMLInputElement).value)}
            />
          {/if}
          <div class="type-hint">{input.feel_type}</div>
        </div>
      {/each}
    </div>
  {/if}

  <!-- Error -->
  {#if error}
    <div class="error">{error}</div>
  {/if}

  <!-- Evaluate Button -->
  <div class="section footer">
    <button class="evaluate-btn" on:click={evaluate} disabled={loading}>
      {loading ? 'Evaluating...' : 'Evaluate'}
    </button>
  </div>
</div>

<style>
  .panel {
    width: 260px;
    background: #161b22;
    border-right: 1px solid #30363d;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }
  .section {
    padding: 12px;
    border-bottom: 1px solid #30363d;
  }
  .footer {
    margin-top: auto;
    border-top: 1px solid #30363d;
    border-bottom: none;
  }
  .label {
    color: #8b949e;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 1px;
    margin-bottom: 8px;
    display: block;
  }
  .field {
    margin-bottom: 12px;
  }
  .field-label {
    color: #c9d1d9;
    font-size: 11px;
    display: block;
    margin-bottom: 4px;
  }
  .type-hint {
    color: #484f58;
    font-size: 10px;
    margin-top: 2px;
  }
  .select, .input {
    width: 100%;
    background: #0d1117;
    border: 1px solid #30363d;
    border-radius: 6px;
    padding: 8px 10px;
    color: #58a6ff;
    font-size: 13px;
    font-family: monospace;
    box-sizing: border-box;
  }
  .toggle {
    color: #58a6ff;
    font-size: 13px;
    font-family: monospace;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .error {
    color: #f85149;
    font-size: 11px;
    padding: 8px 12px;
  }
  .evaluate-btn {
    width: 100%;
    background: #238636;
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 10px;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
  }
  .evaluate-btn:hover {
    background: #2ea043;
  }
  .evaluate-btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }
</style>
```

- [ ] **Step 2: Commit**

```bash
git add viewer/src/lib/components/InputPanel.svelte
git commit -m "feat(viewer): add input panel component

Auto-generates typed input fields from model metadata. Supports
number, string, boolean, and enum (dropdown) inputs."
```

---

## Task 15: DAG Canvas and Main Page

**Files:**
- Create: `viewer/src/lib/components/DagCanvas.svelte`
- Create: `viewer/src/routes/+page.svelte`
- Create: `viewer/src/app.css`

- [ ] **Step 1: Create DagCanvas**

Create `viewer/src/lib/components/DagCanvas.svelte`:

```svelte
<script lang="ts">
  import { SvelteFlow, MiniMap, Controls, Background } from '@xyflow/svelte';
  import '@xyflow/svelte/dist/style.css';
  import { traceData } from '$lib/stores';
  import { computeLayout } from '$lib/layout';
  import InputDataNode from './InputDataNode.svelte';
  import DecisionTableNode from './DecisionTableNode.svelte';
  import AnimatedEdge from './AnimatedEdge.svelte';
  import StepControls from './StepControls.svelte';

  const nodeTypes = {
    inputData: InputDataNode,
    decisionTable: DecisionTableNode,
  };

  const edgeTypes = {
    animated: AnimatedEdge,
  };

  let nodes: any[] = [];
  let edges: any[] = [];

  $: if ($traceData) {
    const layout = computeLayout($traceData.graph);
    nodes = layout.nodes;
    edges = layout.edges;
  }
</script>

<div class="canvas">
  {#if $traceData}
    <div class="step-controls-overlay">
      <StepControls />
    </div>
    <SvelteFlow
      {nodes}
      {edges}
      {nodeTypes}
      {edgeTypes}
      fitView
      minZoom={0.2}
      maxZoom={2}
    >
      <Background />
      <Controls />
      <MiniMap />
    </SvelteFlow>
  {:else}
    <div class="empty">
      <p>Select a model and click Evaluate to visualize the decision graph.</p>
    </div>
  {/if}
</div>

<style>
  .canvas {
    flex: 1;
    position: relative;
    background: #0d1117;
  }
  .step-controls-overlay {
    position: absolute;
    top: 12px;
    left: 12px;
    z-index: 10;
  }
  .empty {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    color: #484f58;
    font-size: 14px;
  }
</style>
```

- [ ] **Step 2: Create global styles**

Create `viewer/src/app.css`:

```css
:root {
  --bg-primary: #0d1117;
  --bg-secondary: #161b22;
  --border: #30363d;
  --text-primary: #c9d1d9;
  --text-secondary: #8b949e;
  --accent-blue: #58a6ff;
  --accent-green: #3fb950;
  --accent-yellow: #d29922;
  --accent-red: #f85149;
}

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif;
  font-size: 14px;
}
```

- [ ] **Step 3: Create main page**

Create `viewer/src/routes/+page.svelte`:

```svelte
<script lang="ts">
  import InputPanel from '$lib/components/InputPanel.svelte';
  import DagCanvas from '$lib/components/DagCanvas.svelte';
  import '../app.css';
</script>

<div class="layout">
  <InputPanel />
  <DagCanvas />
</div>

<style>
  .layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }
</style>
```

- [ ] **Step 4: Verify dev server runs without errors**

```bash
cd viewer && npm run dev
```

Expected: Dev server starts. Page loads showing the empty state ("Select a model and click Evaluate...").

- [ ] **Step 5: Commit**

```bash
git add viewer/src/lib/components/DagCanvas.svelte viewer/src/routes/+page.svelte viewer/src/app.css
git commit -m "feat(viewer): add DAG canvas and main page layout

Three-panel layout with InputPanel (left) and DagCanvas (center).
Dark theme matching GitHub's color scheme. Svelte Flow with custom
node/edge types and step-through controls overlay."
```

---

## Task 16: Integration Test — End to End

**Files:**
- No new files — this task verifies the full pipeline works.

- [ ] **Step 1: Start the dsntk server with a test model**

Find a DMN test model in the project that has decision tables. Check `examples/` or `bbt/` for suitable files.

```bash
cargo +stable build --workspace
./target/debug/dsntk srv examples/
```

(Adjust the path to point to a directory with `.dmn` files.)

- [ ] **Step 2: Test the models endpoint**

```bash
curl http://localhost:8080/api/v1/models | jq .
```

Expected: JSON response with model list (may be empty if `get_models` handler still returns empty — see Task 6 note).

- [ ] **Step 3: Test the evaluate-trace endpoint**

```bash
curl -X POST http://localhost:8080/api/v1/evaluate-trace \
  -H 'Content-Type: application/json' \
  -d '{"namespace":"...","invocable":"...","inputs":{...}}' | jq .
```

Use the namespace and invocable name from the model you loaded. Expected: JSON with `result` and `trace` fields.

- [ ] **Step 4: Start the viewer and verify it connects**

```bash
cd viewer && npm run dev
```

Open http://localhost:5173 in a browser. Select the model, enter inputs, click Evaluate. The DAG should render with nodes and edges.

- [ ] **Step 5: Test step-through animation**

Use the step controls to step forward through the evaluation. Verify:
- Input nodes start blue, turn green when their downstream decision evaluates
- Decision table nodes start grey/dashed, turn yellow when evaluating, turn green when done
- Matched rows highlight green within decision table nodes
- Edges animate with a pulse and show value labels
- Step counter updates correctly

- [ ] **Step 6: Test live re-evaluation**

Change an input value and click Evaluate again. Verify the DAG updates with new matched rows and values.

- [ ] **Step 7: Commit any fixes**

```bash
git add -A
git commit -m "fix(viewer): integration fixes from end-to-end testing"
```

---

## Summary

| Task | Component | Key Files |
|---|---|---|
| 1 | Trace data structures | `model-evaluator/src/trace.rs` |
| 2 | Graph extraction | `model-evaluator/src/model_definitions.rs` |
| 3 | Cell evaluation capture | `model-evaluator/src/decision_table.rs` |
| 4 | TraceStep collection | `model-evaluator/src/decision.rs` |
| 5 | Traced evaluation API | `model-evaluator/src/model_evaluator.rs` |
| 6 | Server endpoints | `server/src/trace_handlers.rs` |
| 7 | SvelteKit scaffold | `viewer/` |
| 8 | TS types + API client | `viewer/src/lib/types.ts`, `api.ts` |
| 9 | Svelte stores | `viewer/src/lib/stores.ts` |
| 10 | Dagre layout | `viewer/src/lib/layout.ts` |
| 11 | Custom nodes | `DecisionTableNode.svelte`, `InputDataNode.svelte` |
| 12 | Animated edges | `AnimatedEdge.svelte` |
| 13 | Step controls | `StepControls.svelte` |
| 14 | Input panel | `InputPanel.svelte` |
| 15 | Canvas + page | `DagCanvas.svelte`, `+page.svelte` |
| 16 | Integration test | End-to-end verification |
