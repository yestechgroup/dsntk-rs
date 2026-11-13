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
    .input_data(InputData::build(
      Some("amount"),
      "amount",
      None,
      "number",
      Some(FeelType::Number),
    ))
    .business_knowledge_model(BusinessKnowledgeModel::build(
      Some("calculate_double"),
      "calculate_double",
      None,
      "number",
      Some(FeelType::Number),
      vec![InformationItem::build("rate", "number", Some(FeelType::Number))],
      FunctionDefinition::build(
        vec![],
        Some(ExpressionInstance::LiteralExpression(Box::new(
          LiteralExpression::build("rate * 2"),
        ))),
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
          Some(ExpressionInstance::LiteralExpression(Box::new(
            LiteralExpression::build("amount"),
          ))),
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
  let value = evaluator.evaluate_invocable(
    "https://example.com/assessment",
    "Assessment",
    "DoubleIt",
    &input_data,
  );
  match value {
    Value::Number(number) => assert_eq!(number.to_string(), "42"),
    other => panic!("expected the number 42 from the decision service, got: {other:?}"),
  }
}
