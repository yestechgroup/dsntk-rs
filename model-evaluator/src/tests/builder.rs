//! Tests for the programmatic model builder: a model assembled with
//! `DefinitionsBuilder` must evaluate exactly like a parsed model —
//! the builder is a full-construction path, not a toy.

use crate::model_evaluator::ModelEvaluator;
use dsntk_common::HRef;
use dsntk_feel::context::FeelContext;
use dsntk_feel::values::Value;
use dsntk_feel::{FeelType, Name};
use dsntk_model::builder::DefinitionsBuilder;
use dsntk_model::*;

/// Builds a small decision model: one input, one business knowledge
/// model, one decision invoking the model, and one decision service
/// exposing the decision.
fn built_definitions() -> Definitions {
  DefinitionsBuilder::new("Assessment", "https://example.com/assessment")
    .input_data(InputData::build(Some("amount"), "amount", None, "number", Some(FeelType::Number)))
    .business_knowledge_model(BusinessKnowledgeModel::build(
      Some("calculate_double"),
      "calculate_double",
      None,
      "number",
      Some(FeelType::Number),
      vec![InformationItem::build("rate", "number", Some(FeelType::Number))],
      FunctionDefinition::build(
        vec![],
        Some(ExpressionInstance::LiteralExpression(Box::new(LiteralExpression::build("rate * 2")))),
        FunctionKind::Feel,
      ),
    ))
    .decision(Decision::build(
      Some("doubled"),
      "doubled",
      None,
      InformationItem::build("doubled", "number", Some(FeelType::Number)),
      Some(ExpressionInstance::Invocation(Box::new(Invocation::build(
        ExpressionInstance::LiteralExpression(Box::new(LiteralExpression::build("calculate_double"))),
        vec![Binding::build(
          InformationItem::build("rate", "number", Some(FeelType::Number)),
          Some(ExpressionInstance::LiteralExpression(Box::new(LiteralExpression::build("amount")))),
        )],
      )))),
      vec![InformationRequirement::build(None, Some(HRef::try_from("#amount").unwrap()))],
      vec![KnowledgeRequirement::build(HRef::try_from("#calculate_double").unwrap())],
      vec![],
    ))
    .decision_service(DecisionService::build(
      Some("DoubleIt"),
      "DoubleIt",
      None,
      InformationItem::build("DoubleIt", "number", None),
      vec![HRef::try_from("#amount").unwrap()],
      vec![],
      vec![HRef::try_from("#doubled").unwrap()],
      vec![HRef::try_from("#doubled").unwrap(), HRef::try_from("#calculate_double").unwrap()],
    ))
    .build()
}

#[test]
fn built_model_has_the_expected_shape() {
  let definitions = built_definitions();
  assert_eq!(definitions.name(), "Assessment");
  assert_eq!(definitions.input_data().len(), 1);
  assert_eq!(definitions.business_knowledge_models().len(), 1);
  assert_eq!(definitions.decisions().len(), 1);
  assert_eq!(definitions.decision_services().len(), 1);
  let decision = &definitions.decisions()[0];
  assert_eq!(decision.name(), "doubled");
  assert_eq!(decision.information_requirements().len(), 1);
  assert_eq!(decision.knowledge_requirements().len(), 1);
}

#[test]
fn built_model_evaluates_through_the_decision_service() {
  let definitions = built_definitions();
  let evaluator = ModelEvaluator::new(&[definitions]).expect("model evaluator builds");
  let mut input_data = FeelContext::default();
  input_data.set_entry(&Name::from("amount"), Value::Number("21".parse().unwrap()));
  let value = evaluator.evaluate_invocable("https://example.com/assessment", "Assessment", "DoubleIt", &input_data);
  match value {
    Value::Number(number) => assert_eq!(number.to_string(), "42"),
    other => panic!("expected the number 42 from the decision service, got: {other:?}"),
  }
}

/// Builds a table-driven model: one input, one decision whose logic is a
/// two-rule decision table (unique hit policy), and a service exposing it.
/// Rule 0: amount < 100 → "low". Rule 1: amount >= 100 → "high".
fn built_table_definitions() -> Definitions {
  use dsntk_model::{DecisionRule, DecisionTable, DecisionTableOrientation, HitPolicy, InputClause, InputEntry, OutputClause, OutputEntry};
  let decision_table = DecisionTable::build(
    Some("band_decision"),
    None,
    vec![InputClause {
      input_expression: "amount".to_string(),
      allowed_input_values: None,
    }],
    vec![OutputClause {
      type_ref: None,
      name: Some("band".to_string()),
      allowed_output_values: None,
      default_output_entry: None,
    }],
    vec![],
    vec![
      DecisionRule {
        input_entries: vec![InputEntry { text: "< 100".to_string() }],
        output_entries: vec![OutputEntry { text: "\"low\"".to_string() }],
        annotation_entries: vec![],
      },
      DecisionRule {
        input_entries: vec![InputEntry { text: ">= 100".to_string() }],
        output_entries: vec![OutputEntry { text: "\"high\"".to_string() }],
        annotation_entries: vec![],
      },
    ],
    HitPolicy::Unique,
    None,
    DecisionTableOrientation::RuleAsRow,
    None,
  );
  DefinitionsBuilder::new("Bands", "https://example.com/bands")
    .input_data(InputData::build(Some("amount"), "amount", None, "number", Some(FeelType::Number)))
    .decision(Decision::build(
      Some("band_decision"),
      "band_decision",
      None,
      InformationItem::build("band_decision", "string", Some(FeelType::String)),
      Some(ExpressionInstance::DecisionTable(Box::new(decision_table))),
      vec![InformationRequirement::build(None, Some(HRef::try_from("#amount").unwrap()))],
      vec![],
      vec![],
    ))
    .decision_service(DecisionService::build(
      Some("BandIt"),
      "BandIt",
      None,
      InformationItem::build("BandIt", "string", None),
      vec![HRef::try_from("#amount").unwrap()],
      vec![],
      vec![HRef::try_from("#band_decision").unwrap()],
      // The output decisions are a subset of the encapsulated ones — the
      // builder passes only the NOT-exposed encapsulated decisions, so a
      // decision never evaluates twice within one service invocation.
      vec![],
    ))
    .build()
}

#[test]
fn table_traces_record_the_matching_rule_numbers() {
  let definitions = built_table_definitions();
  let evaluator = ModelEvaluator::new(&[definitions]).expect("model evaluator builds");
  for (amount, expected_rule) in [("5", 0_usize), ("500", 1_usize)] {
    let mut input_data = FeelContext::default();
    input_data.set_entry(&Name::from("amount"), Value::Number(amount.parse().unwrap()));
    let (value, traces) = evaluator.evaluate_invocable_traced("https://example.com/bands", "Bands", "BandIt", &input_data);
    assert!(matches!(value, Value::String(_)), "expected a string band, got: {value:?}");
    assert_eq!(traces.len(), 1, "exactly one decision table evaluated");
    assert_eq!(traces[0].table_id, "band_decision");
    assert_eq!(traces[0].matched_rules, vec![expected_rule], "amount = {amount}");
  }
}
