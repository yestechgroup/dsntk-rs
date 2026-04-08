//! Regression tests for evaluating decision tables parsed from markdown.
//!
//! These tests verify that `evaluate_decision_table_with_trace` correctly
//! handles various hit policies and FEEL name patterns (including multi-word
//! names) when the table is recognized from markdown format.

use crate::evaluate_decision_table_with_trace;
use dsntk_feel::context::FeelContext;
use dsntk_feel::values::Value;
use dsntk_feel::{FeelScope, Name};

/// Helper: parse a markdown decision table body and evaluate it with the given scope.
fn eval_markdown(md_body: &str, scope: &FeelScope) -> crate::DecisionTableEvalResult {
  let recognized = dsntk_recognizer::from_markdown(md_body, false).expect("failed to parse markdown table");
  let model_dt: dsntk_model::DecisionTable = recognized.into();
  evaluate_decision_table_with_trace(scope, &model_dt).expect("evaluation failed")
}

/// Helper: build a FeelScope with entries from a list of (name, value) pairs.
fn scope_with(entries: &[(&str, Value)]) -> FeelScope {
  let scope = FeelScope::default();
  let mut ctx = FeelContext::default();
  for (name, value) in entries {
    ctx.set_entry(&Name::new(&[name]), value.clone());
  }
  scope.push(ctx);
  scope
}

/// Helper: parse a string as a FeelNumber.
fn num(s: &str) -> Value {
  Value::Number(s.parse().expect("invalid number"))
}

/// Helper: string value.
fn str_val(s: &str) -> Value {
  Value::String(s.to_string())
}

// ---------------------------------------------------------------------------
// FIRST hit policy with multi-word FEEL names
// ---------------------------------------------------------------------------

const CREDIT_GRADE_TABLE: &str = r#"
> # Credit Score Classification
> Credit Grade

| F  | Bureau Score | Years Trading | Credit Grade                |
|:--:|:------------:|:-------------:|:---------------------------:|
|    |              |               | "A","B","C","D","Decline"   |
|    | `in`         | `in`          | `out`                       |
|  1 | <30          | -             | "Decline"                   |
|  2 | >=80         | >=5           | "A"                         |
|  3 | [65..80)     | >=3           | "B"                         |
|  4 | [50..65)     | >=2           | "C"                         |
|  5 | -            | -             | "D"                         |
"#;

/// FIRST hit policy: high score, long trading → "A".
#[test]
fn _0001() {
  let scope = scope_with(&[("Bureau Score", num("92")), ("Years Trading", num("12"))]);
  let result = eval_markdown(CREDIT_GRADE_TABLE, &scope);
  assert_eq!(result.value, str_val("A"));
  assert!(result.matched_rules.contains(&1), "rule index 1 should match");
}

/// FIRST hit policy: low score → "Decline".
#[test]
fn _0002() {
  let scope = scope_with(&[("Bureau Score", num("20")), ("Years Trading", num("5"))]);
  let result = eval_markdown(CREDIT_GRADE_TABLE, &scope);
  assert_eq!(result.value, str_val("Decline"));
  assert!(result.matched_rules.contains(&0), "rule index 0 should match");
}

/// FIRST hit policy: catch-all fires when no specific rule matches.
#[test]
fn _0003() {
  let scope = scope_with(&[("Bureau Score", num("45")), ("Years Trading", num("1"))]);
  let result = eval_markdown(CREDIT_GRADE_TABLE, &scope);
  assert_eq!(result.value, str_val("D"));
  assert!(result.matched_rules.contains(&4), "catch-all rule index 4 should match");
}

// ---------------------------------------------------------------------------
// COLLECT hit policy with compound outputs
// ---------------------------------------------------------------------------

const RATE_ADJUSTMENTS_TABLE: &str = r#"
> # Rate Adjustments
> Rate Adjustment

| C  | Sector         | Has Guarantee | Adjustment | Reason Code |
|:--:|:--------------:|:------------:|:----------:|:-----------:|
|    | `in`           | `in`         | `out`      | `out`       |
|  1 | "Construction" | -            | 0.50       | "SEC-01"    |
|  2 | "Hospitality"  | -            | 0.40       | "SEC-02"    |
|  3 | -              | true         | -0.25      | "GTR-01"    |
"#;

/// COLLECT: multiple rules fire, compound outputs returned as list.
#[test]
fn _0004() {
  let scope = scope_with(&[("Sector", str_val("Construction")), ("Has Guarantee", Value::Boolean(true))]);
  let result = eval_markdown(RATE_ADJUSTMENTS_TABLE, &scope);
  // Both SEC-01 and GTR-01 should match.
  assert_eq!(result.matched_rules.len(), 2);
  assert!(result.matched_rules.contains(&0));
  assert!(result.matched_rules.contains(&2));
  // Output should be a list of contexts.
  match &result.value {
    Value::List(items) => assert_eq!(items.len(), 2),
    other => panic!("expected list, got: {other}"),
  }
}

/// COLLECT: no rules match → null.
#[test]
fn _0005() {
  let scope = scope_with(&[("Sector", str_val("Technology")), ("Has Guarantee", Value::Boolean(false))]);
  let result = eval_markdown(RATE_ADJUSTMENTS_TABLE, &scope);
  assert!(result.matched_rules.is_empty());
}

// ---------------------------------------------------------------------------
// Single-word output name (no spaces) works in FEEL evaluation
// ---------------------------------------------------------------------------

const SINGLE_WORD_OUTPUT_TABLE: &str = r#"
> # Sector Risk
> SectorRisk

| F  | Sector       | SectorRisk |
|:--:|:------------:|:----------:|
|    | `in`         | `out`      |
|  1 | "Technology" | "Low"      |
|  2 | "Retail"     | "Medium"   |
|  3 | -            | "High"     |
"#;

/// Single-word output name evaluates without parse errors.
#[test]
fn _0006() {
  let scope = scope_with(&[("Sector", str_val("Technology"))]);
  let result = eval_markdown(SINGLE_WORD_OUTPUT_TABLE, &scope);
  assert_eq!(result.value, str_val("Low"));
}

// ---------------------------------------------------------------------------
// Numeric comparisons in FEEL (ranges, inequalities)
// ---------------------------------------------------------------------------

const LEVERAGE_TABLE: &str = r#"
> # Leverage Check
> Risk

| F  | Leverage Ratio | Risk     |
|:--:|:--------------:|:--------:|
|    | `in`           | `out`    |
|  1 | <=0.50         | "Low"    |
|  2 | (0.50..0.75]   | "Medium" |
|  3 | >0.75          | "High"   |
"#;

/// Numeric range: low leverage.
#[test]
fn _0007() {
  let scope = scope_with(&[("Leverage Ratio", num("0.089"))]);
  let result = eval_markdown(LEVERAGE_TABLE, &scope);
  assert_eq!(result.value, str_val("Low"));
  assert!(result.matched_rules.contains(&0));
}

/// Numeric range: medium leverage.
#[test]
fn _0008() {
  let scope = scope_with(&[("Leverage Ratio", num("0.65"))]);
  let result = eval_markdown(LEVERAGE_TABLE, &scope);
  assert_eq!(result.value, str_val("Medium"));
}

/// Numeric range: high leverage.
#[test]
fn _0009() {
  let scope = scope_with(&[("Leverage Ratio", num("1.5"))]);
  let result = eval_markdown(LEVERAGE_TABLE, &scope);
  assert_eq!(result.value, str_val("High"));
}

// ---------------------------------------------------------------------------
// PRIORITY hit policy with allowed output values
// ---------------------------------------------------------------------------

const PRIORITY_TABLE: &str = r#"
> # Pricing
> Tier

| P  | Grade | Tier                 |
|:--:|:-----:|:--------------------:|
|    |       | "T1","T2","T3","T4"  |
|    | `in`  | `out`                |
|  1 | "A"   | "T1"                 |
|  2 | "B"   | "T2"                 |
|  3 | "A"   | "T2"                 |
|  4 | -     | "T4"                 |
"#;

/// PRIORITY: "A" matches rules 1 and 3; priority selects "T1" (earlier in allowed values).
#[test]
fn _0010() {
  let scope = scope_with(&[("Grade", str_val("A"))]);
  let result = eval_markdown(PRIORITY_TABLE, &scope);
  assert_eq!(result.value, str_val("T1"));
}
