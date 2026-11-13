//! Programmatic construction of DMN [Definitions] — the public builder API.
//!
//! The `dsntk-model` structures are parsed from XML by [parse](crate::parse),
//! and their fields are internal to this crate. This module exposes the
//! construction path for tools that build decision models in code — the
//! same shapes the parser produces, assembled field by field:
//!
//! - [DefinitionsBuilder] collects item definitions and DRG elements and
//!   produces one [Definitions];
//! - every DRG element type (and the expression types a decision or a
//!   business knowledge model may carry) exposes a `build` constructor
//!   taking exactly the parts the builder cannot infer.
//!
//! The namespace and the model name propagate from the builder to every
//! contained element in [DefinitionsBuilder::build], so references made
//! with [HRef] fragments resolve the same way they resolve for parsed
//! models.
//!
//! # Examples
//!
//! ```
//! use dsntk_common::HRef;
//! use dsntk_feel::FeelType;
//! use dsntk_model::builder::DefinitionsBuilder;
//! use dsntk_model::*;
//!
//! let definitions = DefinitionsBuilder::new("Assessment", "https://example.com/assessment")
//!   .input_data(InputData::build(None, "amount", None, "number", Some(FeelType::Number)))
//!   .business_knowledge_model(
//!     BusinessKnowledgeModel::build(
//!       None,
//!       "calculate_esct",
//!       None,
//!       "number",
//!       Some(FeelType::Number),
//!       vec![InformationItem::build("rate", "number", Some(FeelType::Number))],
//!       FunctionDefinition::build(
//!         vec![InformationItem::build("rate", "number", Some(FeelType::Number))],
//!         Some(ExpressionInstance::LiteralExpression(Box::new(
//!           LiteralExpression::build("rate * 2"),
//!         ))),
//!         FunctionKind::Feel,
//!       ),
//!     ),
//!   )
//!   .decision(
//!     Decision::build(
//!       None,
//!       "esct_amount",
//!       None,
//!       InformationItem::build("esct_amount", "number", Some(FeelType::Number)),
//!       Some(ExpressionInstance::LiteralExpression(Box::new(
//!         LiteralExpression::build("rate * 100"),
//!       ))),
//!       vec![],
//!       vec![],
//!       vec![],
//!     ),
//!   )
//!   .decision_service(
//!     DecisionService::build(
//!       None,
//!       "AssessEsct",
//!       None,
//!       InformationItem::build("AssessEsct", "number", None),
//!       vec![],
//!       vec![],
//!       vec![HRef::try_from("#esct_amount").unwrap()],
//!       vec![HRef::try_from("#esct_amount").unwrap()],
//!     ),
//!   )
//!   .build();
//! assert_eq!(definitions.name(), "Assessment");
//! assert_eq!(definitions.decisions().len(), 1);
//! assert_eq!(definitions.decision_services().len(), 1);
//! ```

use dsntk_common::{gen_id, Uri};
use dsntk_feel::{FeelType, Name};

use crate::model::{
  AuthorityRequirement, Binding, BuiltinAggregator, BusinessKnowledgeModel, Decision, DecisionRule, DecisionService, DecisionTable, DecisionTableOrientation, Definitions, DmnId,
  DrgElement, ExpressionInstance, FunctionDefinition, FunctionKind, HitPolicy, InformationItem, InformationRequirement, InputClause, InputData, Invocation, ItemDefinition,
  KnowledgeRequirement, KnowledgeSource, LiteralExpression, OutputClause, RuleAnnotationClause, UnaryTests,
};
use dsntk_common::HRef;

/// Builder of one [Definitions] instance.
#[derive(Default)]
pub struct DefinitionsBuilder {
  /// Model name — the [Definitions] element's name.
  name: String,
  /// Model namespace — the [Definitions] element's namespace.
  namespace: Uri,
  /// Item definitions collected so far.
  item_definitions: Vec<ItemDefinition>,
  /// DRG elements collected so far.
  drg_elements: Vec<DrgElement>,
}

impl DefinitionsBuilder {
  /// Creates a builder for a model with the specified name and namespace.
  pub fn new(name: impl Into<String>, namespace: impl Into<String>) -> Self {
    Self {
      name: name.into(),
      namespace: namespace.into(),
      item_definitions: vec![],
      drg_elements: vec![],
    }
  }

  /// Adds an item definition to the model.
  pub fn item_definition(mut self, item_definition: ItemDefinition) -> Self {
    self.item_definitions.push(item_definition);
    self
  }

  /// Adds an input data element to the model.
  pub fn input_data(mut self, input_data: InputData) -> Self {
    self.drg_elements.push(DrgElement::InputData(input_data));
    self
  }

  /// Adds a knowledge source element to the model.
  pub fn knowledge_source(mut self, knowledge_source: KnowledgeSource) -> Self {
    self.drg_elements.push(DrgElement::KnowledgeSource(knowledge_source));
    self
  }

  /// Adds a business knowledge model to the model.
  pub fn business_knowledge_model(mut self, bkm: BusinessKnowledgeModel) -> Self {
    self.drg_elements.push(DrgElement::BusinessKnowledgeModel(bkm));
    self
  }

  /// Adds a decision to the model.
  pub fn decision(mut self, decision: Decision) -> Self {
    self.drg_elements.push(DrgElement::Decision(decision));
    self
  }

  /// Adds a decision service to the model.
  pub fn decision_service(mut self, decision_service: DecisionService) -> Self {
    self.drg_elements.push(DrgElement::DecisionService(decision_service));
    self
  }

  /// Builds the [Definitions], propagating the namespace and the model
  /// name into every contained element.
  pub fn build(self) -> Definitions {
    let mut definitions = Definitions {
      namespace: self.namespace.clone(),
      model_name: self.name.clone(),
      id: DmnId::Generated(gen_id()),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      name: self.name.clone(),
      feel_name: Name::from(self.name),
      expression_language: None,
      type_language: None,
      exporter: None,
      exporter_version: None,
      item_definitions: self.item_definitions,
      drg_elements: self.drg_elements,
      business_context_elements: vec![],
      imports: vec![],
      dmndi: None,
    };
    let namespace = definitions.namespace.clone();
    let model_name = definitions.model_name.clone();
    for item_definition in definitions.item_definitions.iter_mut() {
      item_definition.namespace = namespace.clone();
      item_definition.model_name = model_name.clone();
      Self::propagate_item_definition(item_definition, &namespace, &model_name);
    }
    for drg_element in definitions.drg_elements.iter_mut() {
      Self::propagate_drg_element(drg_element, &namespace, &model_name);
    }
    definitions
  }

  /// Propagates the namespace and the model name into an item definition
  /// and its nested components.
  fn propagate_item_definition(item_definition: &mut ItemDefinition, namespace: &str, model_name: &str) {
    for component in item_definition.item_components.iter_mut() {
      component.namespace = namespace.to_string();
      component.model_name = model_name.to_string();
      Self::propagate_item_definition(component, namespace, model_name);
    }
  }

  /// Propagates the namespace and the model name into a DRG element and
  /// every requirement it carries.
  fn propagate_drg_element(drg_element: &mut DrgElement, namespace: &str, model_name: &str) {
    match drg_element {
      DrgElement::Decision(decision) => {
        decision.namespace = namespace.to_string();
        decision.model_name = model_name.to_string();
        for requirement in decision.information_requirements.iter_mut() {
          requirement.namespace = namespace.to_string();
          requirement.model_name = model_name.to_string();
        }
        for requirement in decision.knowledge_requirements.iter_mut() {
          requirement.namespace = namespace.to_string();
          requirement.model_name = model_name.to_string();
        }
        for requirement in decision.authority_requirements.iter_mut() {
          requirement.namespace = namespace.to_string();
          requirement.model_name = model_name.to_string();
        }
        if let Some(logic) = decision.decision_logic.as_mut() {
          Self::propagate_expression(logic, namespace, model_name);
        }
      }
      DrgElement::InputData(input_data) => {
        input_data.namespace = namespace.to_string();
        input_data.model_name = model_name.to_string();
      }
      DrgElement::BusinessKnowledgeModel(bkm) => {
        bkm.namespace = namespace.to_string();
        bkm.model_name = model_name.to_string();
        for requirement in bkm.knowledge_requirements.iter_mut() {
          requirement.namespace = namespace.to_string();
          requirement.model_name = model_name.to_string();
        }
        for requirement in bkm.authority_requirements.iter_mut() {
          requirement.namespace = namespace.to_string();
          requirement.model_name = model_name.to_string();
        }
        if let Some(logic) = bkm.encapsulated_logic.as_mut() {
          logic.namespace = namespace.to_string();
          logic.model_name = model_name.to_string();
        }
      }
      DrgElement::DecisionService(decision_service) => {
        decision_service.namespace = namespace.to_string();
        decision_service.model_name = model_name.to_string();
      }
      DrgElement::KnowledgeSource(knowledge_source) => {
        knowledge_source.namespace = namespace.to_string();
        knowledge_source.model_name = model_name.to_string();
        for requirement in knowledge_source.authority_requirements.iter_mut() {
          requirement.namespace = namespace.to_string();
          requirement.model_name = model_name.to_string();
        }
      }
    }
  }

  /// Propagates the namespace and the model name into an expression
  /// instance (recursively, so nested expressions stay consistent).
  fn propagate_expression(expression: &mut ExpressionInstance, namespace: &str, model_name: &str) {
    match expression {
      ExpressionInstance::Conditional(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::Context(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::DecisionTable(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::Every(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::Filter(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::For(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::FunctionDefinition(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::Invocation(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::List(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::LiteralExpression(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::Relation(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
      ExpressionInstance::Some(inner) => {
        inner.namespace = namespace.to_string();
        inner.model_name = model_name.to_string();
      }
    }
  }
}

/// Builds the full [InformationItem] shape (the information item is a
/// named DMN element — it carries the common element attributes).
fn information_item(name: impl Into<String>, type_ref: impl Into<String>, feel_type: Option<FeelType>) -> InformationItem {
  let name = name.into();
  InformationItem {
    namespace: "".to_string(),
    model_name: "".to_string(),
    id: DmnId::Generated(gen_id()),
    description: None,
    label: None,
    extension_elements: vec![],
    extension_attributes: vec![],
    feel_name: Name::from(name.clone()),
    name,
    type_ref: type_ref.into(),
    feel_type,
  }
}

/// Generates an identifier: the provided one, or a fresh unique one.
fn identifier(id: Option<&str>) -> DmnId {
  match id {
    Some(id) => DmnId::Provided(id.to_string()),
    None => DmnId::Generated(gen_id()),
  }
}

impl DecisionTable {
  /// Builds a decision table with the specified identifier and parts —
  /// the same shape [`DecisionTable::new`] produces, with the identifier
  /// under the builder's control (provenance traces report the table's
  /// identifier, so a table lowered for one decision carries that
  /// decision's identifier).
  #[allow(clippy::too_many_arguments)]
  pub fn build(
    id: Option<&str>,
    information_item_name: Option<String>,
    input_clauses: Vec<InputClause>,
    output_clauses: Vec<OutputClause>,
    annotations: Vec<RuleAnnotationClause>,
    rules: Vec<DecisionRule>,
    hit_policy: HitPolicy,
    aggregation: Option<BuiltinAggregator>,
    preferred_orientation: DecisionTableOrientation,
    output_label: Option<String>,
  ) -> Self {
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: identifier(id),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      type_ref: None,
      information_item_name,
      input_clauses,
      output_clauses,
      annotations,
      rules,
      hit_policy,
      aggregation,
      preferred_orientation,
      output_label,
    }
  }
}

impl InformationItem {
  /// Builds an information item with the specified name and type.
  pub fn build(name: impl Into<String>, type_ref: impl Into<String>, feel_type: Option<FeelType>) -> Self {
    information_item(name, type_ref, feel_type)
  }
}

impl InputData {
  /// Builds an input data element with the specified identifier, name,
  /// description, and type.
  pub fn build(id: Option<&str>, name: impl Into<String>, description: Option<&str>, type_ref: impl Into<String>, feel_type: Option<FeelType>) -> Self {
    let name = name.into();
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: identifier(id),
      description: description.map(|description| description.to_string()),
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      feel_name: Name::from(name.clone()),
      name: name.clone(),
      variable: information_item(name, type_ref, feel_type),
    }
  }
}

impl KnowledgeSource {
  /// Builds a knowledge source with the specified identifier, name, and
  /// description.
  pub fn build(id: Option<&str>, name: impl Into<String>, description: Option<&str>) -> Self {
    let name = name.into();
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: identifier(id),
      description: description.map(|description| description.to_string()),
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      feel_name: Name::from(name.clone()),
      name,
      authority_requirements: vec![],
    }
  }
}

impl ItemDefinition {
  /// Builds an item definition with the specified name, type, allowed
  /// values, and nested components. A component-type item definition (the
  /// named outputs of a decision) carries no type reference — `None` —
  /// and only its components.
  pub fn build(
    id: Option<&str>,
    name: impl Into<String>,
    type_ref: Option<&str>,
    feel_type: Option<FeelType>,
    allowed_values: Option<UnaryTests>,
    item_components: Vec<ItemDefinition>,
    is_collection: bool,
  ) -> Self {
    let name = name.into();
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: identifier(id),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      feel_name: Name::from(name.clone()),
      name,
      type_language: None,
      feel_type,
      allowed_values,
      item_components,
      is_collection,
      function_item: None,
      type_ref: type_ref.map(|type_ref| type_ref.to_string()),
    }
  }
}

impl UnaryTests {
  /// Builds unary tests from the specified text.
  pub fn build(text: impl Into<String>) -> Self {
    Self {
      text: Some(text.into()),
      expression_language: None,
    }
  }
}

impl LiteralExpression {
  /// Builds a literal expression from the specified text.
  pub fn build(text: impl Into<String>) -> Self {
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: DmnId::Generated(gen_id()),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      text: Some(text.into()),
      expression_language: None,
      imported_values: None,
      type_ref: None,
    }
  }
}

impl Invocation {
  /// Builds an invocation of the specified function with the specified
  /// parameter bindings.
  pub fn build(called_function: ExpressionInstance, bindings: Vec<Binding>) -> Self {
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: DmnId::Generated(gen_id()),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      called_function,
      bindings,
      type_ref: None,
    }
  }
}

impl Binding {
  /// Builds a binding of the specified parameter to the specified
  /// binding formula.
  pub fn build(parameter: InformationItem, binding_formula: Option<ExpressionInstance>) -> Self {
    Self { parameter, binding_formula }
  }
}

impl FunctionDefinition {
  /// Builds a function definition with the specified parameters, body,
  /// and kind.
  pub fn build(formal_parameters: Vec<InformationItem>, body: Option<ExpressionInstance>, kind: FunctionKind) -> Self {
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: DmnId::Generated(gen_id()),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      formal_parameters,
      body,
      kind,
      type_ref: None,
    }
  }
}

impl BusinessKnowledgeModel {
  /// Builds a business knowledge model with the specified identifier,
  /// name, description, output type, formal parameters, and the
  /// encapsulated logic (the function the model applies).
  #[allow(clippy::too_many_arguments)]
  pub fn build(
    id: Option<&str>,
    name: impl Into<String>,
    description: Option<&str>,
    output_type_ref: impl Into<String>,
    output_feel_type: Option<FeelType>,
    formal_parameters: Vec<InformationItem>,
    encapsulated_logic: FunctionDefinition,
  ) -> Self {
    let mut encapsulated_logic = encapsulated_logic;
    encapsulated_logic.formal_parameters = formal_parameters;
    let name = name.into();
    let variable = information_item(name.clone(), output_type_ref, output_feel_type);
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: identifier(id),
      description: description.map(|description| description.to_string()),
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      feel_name: Name::from(name.clone()),
      name,
      variable,
      encapsulated_logic: Some(encapsulated_logic),
      knowledge_requirements: vec![],
      authority_requirements: vec![],
    }
  }
}

impl Decision {
  /// Builds a decision with the specified identifier, name, description,
  /// output variable, decision logic, and requirements.
  #[allow(clippy::too_many_arguments)]
  pub fn build(
    id: Option<&str>,
    name: impl Into<String>,
    description: Option<&str>,
    variable: InformationItem,
    decision_logic: Option<ExpressionInstance>,
    information_requirements: Vec<InformationRequirement>,
    knowledge_requirements: Vec<KnowledgeRequirement>,
    authority_requirements: Vec<AuthorityRequirement>,
  ) -> Self {
    let name = name.into();
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: identifier(id),
      description: description.map(|description| description.to_string()),
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      feel_name: Name::from(name.clone()),
      name,
      question: None,
      allowed_answers: None,
      variable,
      decision_logic,
      information_requirements,
      knowledge_requirements,
      authority_requirements,
    }
  }
}

impl InformationRequirement {
  /// Builds an information requirement referencing the required decision
  /// or the required input data (exactly one of them is expected).
  pub fn build(required_decision: Option<HRef>, required_input: Option<HRef>) -> Self {
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: DmnId::Generated(gen_id()),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      required_decision,
      required_input,
    }
  }
}

impl KnowledgeRequirement {
  /// Builds a knowledge requirement referencing the required business
  /// knowledge model.
  pub fn build(required_knowledge: HRef) -> Self {
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: DmnId::Generated(gen_id()),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      required_knowledge,
    }
  }
}

impl AuthorityRequirement {
  /// Builds an authority requirement referencing the required knowledge
  /// source, decision, or input data.
  pub fn build(required_authority: Option<HRef>, required_decision: Option<HRef>, required_input: Option<HRef>) -> Self {
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: DmnId::Generated(gen_id()),
      description: None,
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      required_authority,
      required_decision,
      required_input,
    }
  }
}

impl DecisionService {
  /// Builds a decision service with the specified identifier, name,
  /// description, and the collections of referenced elements.
  #[allow(clippy::too_many_arguments)]
  pub fn build(
    id: Option<&str>,
    name: impl Into<String>,
    description: Option<&str>,
    variable: InformationItem,
    input_data: Vec<HRef>,
    input_decisions: Vec<HRef>,
    output_decisions: Vec<HRef>,
    encapsulated_decisions: Vec<HRef>,
  ) -> Self {
    let name = name.into();
    Self {
      namespace: "".to_string(),
      model_name: "".to_string(),
      id: identifier(id),
      description: description.map(|description| description.to_string()),
      label: None,
      extension_elements: vec![],
      extension_attributes: vec![],
      feel_name: Name::from(name.clone()),
      name,
      variable,
      output_decisions,
      encapsulated_decisions,
      input_decisions,
      input_data,
    }
  }
}
