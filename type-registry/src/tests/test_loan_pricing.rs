//! # Loan pricing and credit decisioning integration tests
//!
//! Models a complete 6-layer loan pricing DRD (Decision Requirements Diagram)
//! that stress-tests the type registry across the full depth of a real-world
//! credit decisioning workflow.
//!
//! ## Decision hierarchy
//!
//! ```text
//!   L1  Raw Inputs ─────────────────────────────────────────────────────────
//!       (income, liabilities, assets, property_value, bureau_score,
//!        loan_amount, term, purpose)
//!
//!   L2  Input Classifiers ──────────────────────────────────────────────────
//!       BorrowerType    ProductType    Jurisdiction    RelationshipTier
//!       (input-data)    (input-data)   (input-data)    (input-data)
//!             │               │              │               │
//!   L3  Sub-Scores ─────────────────────────────────────────────────────────
//!       CreditBanding   Affordability      CollateralLTV
//!       (bkm)           (bkm)              (bkm)
//!             │               │              │
//!   L4  Composite ──────────────────────────────────────────────────────────
//!       CompositeRiskRating ◄──┤             PolicyExceptions
//!       (decision)             │             (decision)
//!                              │                  │
//!   L5  Pricing & Limits ───────────────────────────────────────────────────
//!       PricingMargin                        LimitAndTerm
//!       (decision)                           (decision)
//!             │                                   │
//!   L6  Final Offer ────────────────────────────────────────────────────────
//!       LoanOffer ◄───governed-by───► CreditPolicyFramework (knowledge-source)
//!       (decision)                    BaselIIICapitalRules   (knowledge-source)
//!                                     FCAConsumerDuty        (knowledge-source)
//! ```

use crate::front_matter::{BkmParameter, BkmSignature, DataTypeRef, DmnNode};
use crate::registry::{TypeEntry, TypeRegistry, TypeSource};
use crate::resolver::{resolve_data_type, resolve_field_chain, validate_allowed_value, validate_bkm_signature, validate_link_targets};
use dsntk_feel::{FeelType, Name};
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

// ─── Type builders ──────────────────────────────────────────────────────────

/// Creates a FEEL context type from a list of (field_name, feel_type) pairs.
fn ctx(fields: &[(&str, FeelType)]) -> FeelType {
  let mut map = BTreeMap::new();
  for (name, ft) in fields {
    map.insert(Name::from(*name), ft.clone());
  }
  FeelType::Context(map)
}

/// Creates a TypeEntry with allowed values (enum constraint).
fn enum_entry(name: &str, values: &[&str]) -> TypeEntry {
  TypeEntry {
    name: name.to_string(),
    feel_type: FeelType::String,
    source: TypeSource::Primitive,
    allowed_values: Some(values.iter().map(|v| v.to_string()).collect()),
    optional_fields: HashMap::new(),
  }
}

/// Creates a TypeEntry for a context type with optional fields.
fn context_entry(name: &str, feel_type: FeelType, optional: &[&str]) -> TypeEntry {
  TypeEntry {
    name: name.to_string(),
    feel_type,
    source: TypeSource::Primitive,
    allowed_values: None,
    optional_fields: optional.iter().map(|f| (f.to_string(), true)).collect(),
  }
}

/// Creates a simple TypeEntry with no constraints.
fn simple_entry(name: &str, feel_type: FeelType) -> TypeEntry {
  TypeEntry {
    name: name.to_string(),
    feel_type,
    source: TypeSource::Primitive,
    allowed_values: None,
    optional_fields: HashMap::new(),
  }
}

// ─── L1: Raw application input types ────────────────────────────────────────

fn make_loan_application_type() -> FeelType {
  ctx(&[
    ("gross annual income", FeelType::Number),
    ("total monthly debt", FeelType::Number),
    ("requested monthly repayment", FeelType::Number),
    ("loan amount", FeelType::Number),
    ("term months", FeelType::Number),
    ("purpose", FeelType::String),
    ("property value", FeelType::Number),
    ("assets", FeelType::Number),
  ])
}

fn make_applicant_profile_type() -> FeelType {
  ctx(&[
    ("bureau score", FeelType::Number),
    ("income type", FeelType::String),
    ("verification status", FeelType::String),
    ("existing customer", FeelType::Boolean),
    ("years at address", FeelType::Number),
  ])
}

// ─── L3: Sub-score output types ─────────────────────────────────────────────

fn make_credit_banding_output_type() -> FeelType {
  ctx(&[("credit band", FeelType::String), ("pd estimate", FeelType::Number)])
}

fn make_affordability_output_type() -> FeelType {
  ctx(&[
    ("dti ratio", FeelType::Number),
    ("stressed dti ratio", FeelType::Number),
    ("affordability band", FeelType::String),
    ("reason code", FeelType::String),
    ("calculation basis", FeelType::String),
  ])
}

fn make_collateral_ltv_output_type() -> FeelType {
  ctx(&[("ltv ratio", FeelType::Number), ("ltv band", FeelType::String), ("security coverage", FeelType::String)])
}

// ─── L4: Composite types ───────────────────────────────────────────────────

fn make_composite_risk_output_type() -> FeelType {
  ctx(&[
    ("composite pd band", FeelType::String),
    ("risk rating", FeelType::String),
    (
      "sub score summary",
      ctx(&[("credit", FeelType::String), ("affordability", FeelType::String), ("ltv", FeelType::String)]),
    ),
  ])
}

fn make_policy_exception_output_type() -> FeelType {
  ctx(&[
    ("exception triggered", FeelType::Boolean),
    ("exception code", FeelType::String),
    ("hard stop", FeelType::Boolean),
    ("override permitted", FeelType::Boolean),
  ])
}

// ─── L5: Pricing types ─────────────────────────────────────────────────────

fn make_pricing_output_type() -> FeelType {
  ctx(&[
    ("base rate", FeelType::Number),
    ("margin", FeelType::Number),
    ("all in rate", FeelType::Number),
    ("relationship discount", FeelType::Number),
  ])
}

fn make_limit_term_output_type() -> FeelType {
  ctx(&[("max facility", FeelType::Number), ("max term months", FeelType::Number), ("ltv cap", FeelType::Number)])
}

// ─── L6: Final offer type ──────────────────────────────────────────────────

fn make_loan_offer_output_type() -> FeelType {
  ctx(&[
    ("outcome", FeelType::String),
    ("offered rate", FeelType::Number),
    ("offered amount", FeelType::Number),
    ("offered term months", FeelType::Number),
    (
      "conditions",
      ctx(&[("valuation required", FeelType::Boolean), ("additional documentation", FeelType::String)]),
    ),
    ("decline reason code", FeelType::String),
    (
      "audit trail",
      ctx(&[
        ("credit band", FeelType::String),
        ("affordability band", FeelType::String),
        ("affordability reason", FeelType::String),
        ("ltv band", FeelType::String),
        ("composite risk", FeelType::String),
        ("policy exception", FeelType::String),
        ("income modifier applied", FeelType::String),
      ]),
    ),
  ])
}

// ─── Registry builder ──────────────────────────────────────────────────────

/// Builds the complete loan-pricing type registry: all 6 layers, 16 types.
fn make_loan_pricing_registry() -> TypeRegistry {
  let mut r = TypeRegistry::new();

  // L1 — raw input types
  r.insert(context_entry("LoanApplication", make_loan_application_type(), &["property value", "assets"]))
    .unwrap();
  r.insert(context_entry("ApplicantProfile", make_applicant_profile_type(), &["years at address"])).unwrap();

  // L2 — classifier enums
  r.insert(enum_entry("BorrowerType", &["individual", "sme", "corporate"])).unwrap();
  r.insert(enum_entry("ProductType", &["mortgage", "unsecured term", "revolving facility", "buy-to-let"]))
    .unwrap();
  r.insert(enum_entry("Jurisdiction", &["england-wales", "scotland", "northern-ireland"])).unwrap();
  r.insert(enum_entry("RelationshipTier", &["new", "existing", "premier", "private banking"])).unwrap();

  // L3 — sub-score outputs
  r.insert(enum_entry("CreditBand", &["AAA", "AA", "A", "BBB", "BB", "B", "C", "D"])).unwrap();
  r.insert(enum_entry("AffordabilityBand", &["Pass", "Marginal", "Refer", "Fail"])).unwrap();
  r.insert(enum_entry("LtvBand", &["low", "standard", "high", "very high"])).unwrap();
  r.insert(simple_entry("CreditBandingOutput", make_credit_banding_output_type())).unwrap();
  r.insert(simple_entry("AffordabilityOutput", make_affordability_output_type())).unwrap();
  r.insert(simple_entry("CollateralLtvOutput", make_collateral_ltv_output_type())).unwrap();

  // L4 — composite outputs
  r.insert(simple_entry("CompositeRiskOutput", make_composite_risk_output_type())).unwrap();
  r.insert(context_entry("PolicyExceptionOutput", make_policy_exception_output_type(), &["exception code"]))
    .unwrap();

  // L5 — pricing and limit outputs
  r.insert(simple_entry("PricingOutput", make_pricing_output_type())).unwrap();
  r.insert(simple_entry("LimitTermOutput", make_limit_term_output_type())).unwrap();

  // L6 — final offer
  r.insert(context_entry("LoanOfferOutput", make_loan_offer_output_type(), &["decline reason code", "conditions"]))
    .unwrap();

  // L3 — income verification modifier enum
  r.insert(enum_entry(
    "IncomeModifierCode",
    &[
      "INC-00", "INC-01", "INC-02", "INC-03", "INC-04", "INC-05", "INC-06", "INC-07", "INC-08", "INC-09", "INC-10", "INC-99",
    ],
  ))
  .unwrap();

  // L6 — decision outcome enum
  r.insert(enum_entry("LoanOutcome", &["approve", "refer", "decline"])).unwrap();

  r
}

// ─── DRD node builders ─────────────────────────────────────────────────────

fn input_data_node(id: &str, name: &str, type_ref: &str) -> DmnNode {
  DmnNode {
    id: id.to_string(),
    node_type: "input-data".to_string(),
    name: name.to_string(),
    data_type: Some(DataTypeRef {
      type_ref: type_ref.to_string(),
      schema: None,
    }),
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
    feel_expression: None,
    output_name: None,
  }
}

fn bkm_node(id: &str, name: &str, return_type: &str, params: &[(&str, &str)]) -> DmnNode {
  DmnNode {
    id: id.to_string(),
    node_type: "bkm".to_string(),
    name: name.to_string(),
    data_type: Some(DataTypeRef {
      type_ref: return_type.to_string(),
      schema: None,
    }),
    signature: Some(BkmSignature {
      parameters: params
        .iter()
        .map(|(n, t)| BkmParameter {
          name: n.to_string(),
          param_type: t.to_string(),
        })
        .collect(),
      return_type: Some(return_type.to_string()),
      schema: None,
    }),
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
    feel_expression: None,
    output_name: None,
  }
}

fn decision_node(id: &str, name: &str, type_ref: &str, requires: &[&str], governed_by: &[&str], supported_by: &[&str]) -> DmnNode {
  DmnNode {
    id: id.to_string(),
    node_type: "decision".to_string(),
    name: name.to_string(),
    data_type: Some(DataTypeRef {
      type_ref: type_ref.to_string(),
      schema: None,
    }),
    signature: None,
    uri: None,
    owner: None,
    requires: if requires.is_empty() {
      None
    } else {
      Some(requires.iter().map(|s| s.to_string()).collect())
    },
    governed_by: if governed_by.is_empty() {
      None
    } else {
      Some(governed_by.iter().map(|s| s.to_string()).collect())
    },
    supported_by: if supported_by.is_empty() {
      None
    } else {
      Some(supported_by.iter().map(|s| s.to_string()).collect())
    },
    feel_expression: None,
    output_name: None,
  }
}

fn knowledge_source_node(id: &str, name: &str, uri: &str) -> DmnNode {
  DmnNode {
    id: id.to_string(),
    node_type: "knowledge-source".to_string(),
    name: name.to_string(),
    data_type: None,
    signature: None,
    uri: Some(uri.to_string()),
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
    feel_expression: None,
    output_name: None,
  }
}

/// Builds the complete set of DRD nodes for the loan pricing model.
fn make_loan_pricing_nodes() -> Vec<DmnNode> {
  vec![
    // L1 — raw inputs
    input_data_node("loan_application", "Loan Application", "LoanApplication"),
    input_data_node("applicant_profile", "Applicant Profile", "ApplicantProfile"),
    // L2 — classifiers (modelled as input-data since they translate raw inputs)
    input_data_node("borrower_type", "Borrower Type", "BorrowerType"),
    input_data_node("product_type", "Product Type", "ProductType"),
    input_data_node("jurisdiction", "Jurisdiction", "Jurisdiction"),
    input_data_node("relationship_tier", "Relationship Tier", "RelationshipTier"),
    // L3 — sub-score BKMs
    bkm_node(
      "credit_banding",
      "Credit Banding",
      "CreditBandingOutput",
      &[("bureau score", "number"), ("borrower type", "BorrowerType")],
    ),
    bkm_node(
      "affordability",
      "Affordability Assessment",
      "AffordabilityOutput",
      &[
        ("gross annual income", "number"),
        ("total monthly debt", "number"),
        ("requested monthly repayment", "number"),
        ("stress test rate", "number"),
        ("product type", "ProductType"),
        ("income type", "string"),
        ("verification status", "string"),
      ],
    ),
    bkm_node(
      "collateral_ltv",
      "Collateral LTV",
      "CollateralLtvOutput",
      &[("loan amount", "number"), ("property value", "number"), ("product type", "ProductType")],
    ),
    // L4 — composite decisions
    decision_node(
      "composite_risk",
      "Composite Risk Rating",
      "CompositeRiskOutput",
      &["credit_banding", "affordability", "collateral_ltv"],
      &[],
      &["credit_banding", "affordability", "collateral_ltv"],
    ),
    decision_node(
      "policy_exceptions",
      "Policy Exceptions",
      "PolicyExceptionOutput",
      &["composite_risk", "loan_application"],
      &["credit_policy_framework"],
      &[],
    ),
    // L5 — pricing and limits
    decision_node(
      "pricing_margin",
      "Pricing Margin",
      "PricingOutput",
      &["composite_risk", "product_type", "jurisdiction", "relationship_tier"],
      &[],
      &[],
    ),
    decision_node(
      "limit_and_term",
      "Limit and Term",
      "LimitTermOutput",
      &["composite_risk", "collateral_ltv", "product_type"],
      &[],
      &[],
    ),
    // L6 — final offer
    decision_node(
      "loan_offer",
      "Loan Offer",
      "LoanOfferOutput",
      &["pricing_margin", "limit_and_term", "policy_exceptions"],
      &["credit_policy_framework", "basel_iii", "fca_consumer_duty"],
      &[],
    ),
    // Knowledge sources (regulatory frameworks)
    knowledge_source_node("credit_policy_framework", "Credit Policy Framework", "urn:bank:credit-policy:2024"),
    knowledge_source_node("basel_iii", "Basel III Capital Rules", "https://bis.org/bcbs/basel3"),
    knowledge_source_node("fca_consumer_duty", "FCA Consumer Duty", "https://fca.org.uk/consumer-duty"),
  ]
}

// ═══════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════

// --- Registry completeness (L1–L6) ---

#[test]
fn _0001() {
  // All 19 domain types are registered without conflict
  let registry = make_loan_pricing_registry();
  assert_eq!(registry.len(), 19);
  for name in [
    "LoanApplication",
    "ApplicantProfile",
    "BorrowerType",
    "ProductType",
    "Jurisdiction",
    "RelationshipTier",
    "CreditBand",
    "AffordabilityBand",
    "LtvBand",
    "CreditBandingOutput",
    "AffordabilityOutput",
    "CollateralLtvOutput",
    "CompositeRiskOutput",
    "PolicyExceptionOutput",
    "PricingOutput",
    "LimitTermOutput",
    "LoanOfferOutput",
    "IncomeModifierCode",
    "LoanOutcome",
  ] {
    assert!(registry.get(name).is_some(), "missing type: {name}");
  }
}

// --- L1: raw input field resolution ---

#[test]
fn _0002() {
  // LoanApplication fields resolve to expected primitives
  let registry = make_loan_pricing_registry();
  let app = &registry.get("LoanApplication").unwrap().feel_type;
  assert_eq!(resolve_field_chain(app, &["gross annual income"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(app, &["total monthly debt"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(app, &["purpose"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(app, &["loan amount"]), Some(FeelType::Number));
}

#[test]
fn _0003() {
  // ApplicantProfile fields resolve correctly
  let registry = make_loan_pricing_registry();
  let profile = &registry.get("ApplicantProfile").unwrap().feel_type;
  assert_eq!(resolve_field_chain(profile, &["bureau score"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(profile, &["income type"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(profile, &["existing customer"]), Some(FeelType::Boolean));
}

#[test]
fn _0004() {
  // Optional fields are correctly marked on LoanApplication
  let registry = make_loan_pricing_registry();
  let entry = registry.get("LoanApplication").unwrap();
  assert_eq!(entry.optional_fields.get("property value"), Some(&true));
  assert_eq!(entry.optional_fields.get("assets"), Some(&true));
  assert_eq!(entry.optional_fields.get("gross annual income"), None);
}

#[test]
fn _0005() {
  // Nonexistent field on LoanApplication returns None
  let registry = make_loan_pricing_registry();
  let app = &registry.get("LoanApplication").unwrap().feel_type;
  assert!(resolve_field_chain(app, &["credit score"]).is_none());
}

// --- L2: classifier enum validation ---

#[test]
fn _0006() {
  // BorrowerType enum accepts all valid values
  let registry = make_loan_pricing_registry();
  let bt = registry.get("BorrowerType").unwrap();
  assert!(validate_allowed_value("individual", bt).is_ok());
  assert!(validate_allowed_value("sme", bt).is_ok());
  assert!(validate_allowed_value("corporate", bt).is_ok());
}

#[test]
fn _0007() {
  // BorrowerType enum rejects unknown values
  let registry = make_loan_pricing_registry();
  let bt = registry.get("BorrowerType").unwrap();
  let err = validate_allowed_value("trust", bt).unwrap_err();
  assert!(err.to_string().contains("not in allowed values"));
}

#[test]
fn _0008() {
  // ProductType enum validates all four product categories
  let registry = make_loan_pricing_registry();
  let pt = registry.get("ProductType").unwrap();
  for product in ["mortgage", "unsecured term", "revolving facility", "buy-to-let"] {
    assert!(validate_allowed_value(product, pt).is_ok(), "rejected valid product: {product}");
  }
  assert!(validate_allowed_value("overdraft", pt).is_err());
}

#[test]
fn _0009() {
  // Jurisdiction enum validates UK jurisdictions
  let registry = make_loan_pricing_registry();
  let j = registry.get("Jurisdiction").unwrap();
  assert!(validate_allowed_value("england-wales", j).is_ok());
  assert!(validate_allowed_value("scotland", j).is_ok());
  assert!(validate_allowed_value("northern-ireland", j).is_ok());
  assert!(validate_allowed_value("channel-islands", j).is_err());
}

#[test]
fn _0010() {
  // RelationshipTier enum includes private banking
  let registry = make_loan_pricing_registry();
  let rt = registry.get("RelationshipTier").unwrap();
  assert!(validate_allowed_value("private banking", rt).is_ok());
  assert!(validate_allowed_value("new", rt).is_ok());
  assert!(validate_allowed_value("vip", rt).is_err());
}

// --- L3: sub-score type structure ---

#[test]
fn _0011() {
  // CreditBand enum covers the full rating spectrum (AAA through D)
  let registry = make_loan_pricing_registry();
  let cb = registry.get("CreditBand").unwrap();
  for band in ["AAA", "AA", "A", "BBB", "BB", "B", "C", "D"] {
    assert!(validate_allowed_value(band, cb).is_ok(), "rejected valid band: {band}");
  }
  assert!(validate_allowed_value("E", cb).is_err());
  assert!(validate_allowed_value("aaa", cb).is_err()); // case-sensitive
}

#[test]
fn _0012() {
  // AffordabilityBand enum validates the four outcome bands
  let registry = make_loan_pricing_registry();
  let ab = registry.get("AffordabilityBand").unwrap();
  for band in ["Pass", "Marginal", "Refer", "Fail"] {
    assert!(validate_allowed_value(band, ab).is_ok(), "rejected valid band: {band}");
  }
  assert!(validate_allowed_value("pass", ab).is_err()); // case-sensitive
  assert!(validate_allowed_value("Decline", ab).is_err());
}

#[test]
fn _0013() {
  // AffordabilityOutput has all required fields including calculation basis
  let registry = make_loan_pricing_registry();
  let ao = &registry.get("AffordabilityOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(ao, &["dti ratio"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(ao, &["stressed dti ratio"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(ao, &["affordability band"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(ao, &["reason code"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(ao, &["calculation basis"]), Some(FeelType::String));
}

#[test]
fn _0014() {
  // CreditBandingOutput contains credit band and PD estimate
  let registry = make_loan_pricing_registry();
  let cbo = &registry.get("CreditBandingOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(cbo, &["credit band"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(cbo, &["pd estimate"]), Some(FeelType::Number));
}

#[test]
fn _0015() {
  // CollateralLtvOutput contains ltv ratio, ltv band, and security coverage
  let registry = make_loan_pricing_registry();
  let ltv = &registry.get("CollateralLtvOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(ltv, &["ltv ratio"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(ltv, &["ltv band"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(ltv, &["security coverage"]), Some(FeelType::String));
}

#[test]
fn _0016() {
  // IncomeModifierCode enum covers all 12 modifier codes
  let registry = make_loan_pricing_registry();
  let imc = registry.get("IncomeModifierCode").unwrap();
  for code in [
    "INC-00", "INC-01", "INC-02", "INC-03", "INC-04", "INC-05", "INC-06", "INC-07", "INC-08", "INC-09", "INC-10", "INC-99",
  ] {
    assert!(validate_allowed_value(code, imc).is_ok(), "rejected valid code: {code}");
  }
  assert!(validate_allowed_value("INC-11", imc).is_err());
  assert!(validate_allowed_value("AFC-01", imc).is_err());
}

// --- L3: BKM signature validation ---

#[test]
fn _0017() {
  // Credit Banding BKM has valid signature: 2 params, returns CreditBandingOutput
  let nodes = make_loan_pricing_nodes();
  let bkm = nodes.iter().find(|n| n.id == "credit_banding").unwrap();
  assert!(validate_bkm_signature(bkm).is_ok());
  let sig = bkm.signature.as_ref().unwrap();
  assert_eq!(sig.parameters.len(), 2);
  assert_eq!(sig.return_type.as_deref(), Some("CreditBandingOutput"));
}

#[test]
fn _0018() {
  // Affordability BKM has valid signature: 7 params, returns AffordabilityOutput
  let nodes = make_loan_pricing_nodes();
  let bkm = nodes.iter().find(|n| n.id == "affordability").unwrap();
  assert!(validate_bkm_signature(bkm).is_ok());
  let sig = bkm.signature.as_ref().unwrap();
  assert_eq!(sig.parameters.len(), 7);
  assert_eq!(sig.parameters[0].name, "gross annual income");
  assert_eq!(sig.parameters[4].param_type, "ProductType");
  assert_eq!(sig.return_type.as_deref(), Some("AffordabilityOutput"));
}

#[test]
fn _0019() {
  // Collateral LTV BKM has valid signature: 3 params, returns CollateralLtvOutput
  let nodes = make_loan_pricing_nodes();
  let bkm = nodes.iter().find(|n| n.id == "collateral_ltv").unwrap();
  assert!(validate_bkm_signature(bkm).is_ok());
  let sig = bkm.signature.as_ref().unwrap();
  assert_eq!(sig.parameters.len(), 3);
  assert_eq!(sig.return_type.as_deref(), Some("CollateralLtvOutput"));
}

#[test]
fn _0020() {
  // A BKM with no return type fails signature validation
  let broken_bkm = DmnNode {
    id: "broken_affordability".to_string(),
    node_type: "bkm".to_string(),
    name: "Broken Affordability".to_string(),
    data_type: None,
    signature: Some(BkmSignature {
      parameters: vec![BkmParameter {
        name: "income".to_string(),
        param_type: "number".to_string(),
      }],
      return_type: None,
      schema: None,
    }),
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
    feel_expression: None,
    output_name: None,
  };
  let err = validate_bkm_signature(&broken_bkm).unwrap_err();
  assert!(err.to_string().contains("missing a return-type"));
}

#[test]
fn _0021() {
  // A BKM with no signature at all fails validation
  let no_sig_bkm = DmnNode {
    id: "no_sig".to_string(),
    node_type: "bkm".to_string(),
    name: "No Signature BKM".to_string(),
    data_type: None,
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
    feel_expression: None,
    output_name: None,
  };
  let err = validate_bkm_signature(&no_sig_bkm).unwrap_err();
  assert!(err.to_string().contains("missing a return-type"));
}

// --- L3: BKM parameter types resolve through the registry ---

#[test]
fn _0022() {
  // Every BKM parameter type resolves (either as primitive or from registry)
  let registry = make_loan_pricing_registry();
  let nodes = make_loan_pricing_nodes();
  for node in nodes.iter().filter(|n| n.node_type == "bkm") {
    let sig = node.signature.as_ref().unwrap();
    for param in &sig.parameters {
      let data_ref = DataTypeRef {
        type_ref: param.param_type.clone(),
        schema: None,
      };
      let result = resolve_data_type(&data_ref, Path::new("."), &registry);
      assert!(
        result.is_ok(),
        "BKM '{}' param '{}' type '{}' failed to resolve: {}",
        node.name,
        param.name,
        param.param_type,
        result.unwrap_err()
      );
    }
  }
}

#[test]
fn _0023() {
  // Every BKM return type resolves through the registry
  let registry = make_loan_pricing_registry();
  let nodes = make_loan_pricing_nodes();
  for node in nodes.iter().filter(|n| n.node_type == "bkm") {
    let sig = node.signature.as_ref().unwrap();
    let ret = sig.return_type.as_ref().unwrap();
    let data_ref = DataTypeRef {
      type_ref: ret.clone(),
      schema: None,
    };
    let result = resolve_data_type(&data_ref, Path::new("."), &registry);
    assert!(result.is_ok(), "BKM '{}' return type '{}' failed to resolve", node.name, ret);
  }
}

// --- L4: composite risk structure ---

#[test]
fn _0024() {
  // CompositeRiskOutput has nested sub_score_summary with all three sub-score fields
  let registry = make_loan_pricing_registry();
  let cro = &registry.get("CompositeRiskOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(cro, &["composite pd band"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(cro, &["risk rating"]), Some(FeelType::String));
  // nested sub-score summary
  assert_eq!(resolve_field_chain(cro, &["sub score summary", "credit"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(cro, &["sub score summary", "affordability"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(cro, &["sub score summary", "ltv"]), Some(FeelType::String));
}

#[test]
fn _0025() {
  // CompositeRiskOutput: nonexistent sub-score field returns None
  let registry = make_loan_pricing_registry();
  let cro = &registry.get("CompositeRiskOutput").unwrap().feel_type;
  assert!(resolve_field_chain(cro, &["sub score summary", "liquidity"]).is_none());
}

#[test]
fn _0026() {
  // PolicyExceptionOutput has correct fields and optional exception_code
  let registry = make_loan_pricing_registry();
  let pe = registry.get("PolicyExceptionOutput").unwrap();
  assert_eq!(resolve_field_chain(&pe.feel_type, &["exception triggered"]), Some(FeelType::Boolean));
  assert_eq!(resolve_field_chain(&pe.feel_type, &["hard stop"]), Some(FeelType::Boolean));
  assert_eq!(resolve_field_chain(&pe.feel_type, &["override permitted"]), Some(FeelType::Boolean));
  assert_eq!(pe.optional_fields.get("exception code"), Some(&true));
}

// --- L4: Composite Risk decision requires all three L3 sub-scores ---

#[test]
fn _0027() {
  // Composite Risk Rating decision depends on all three L3 BKMs
  let nodes = make_loan_pricing_nodes();
  let composite = nodes.iter().find(|n| n.id == "composite_risk").unwrap();
  let requires = composite.requires.as_ref().unwrap();
  assert!(requires.contains(&"credit_banding".to_string()));
  assert!(requires.contains(&"affordability".to_string()));
  assert!(requires.contains(&"collateral_ltv".to_string()));
}

#[test]
fn _0028() {
  // Composite Risk Rating has supported-by links to its three L3 BKMs
  let nodes = make_loan_pricing_nodes();
  let composite = nodes.iter().find(|n| n.id == "composite_risk").unwrap();
  let all_bkms: Vec<DmnNode> = nodes.iter().filter(|n| n.node_type == "bkm").cloned().collect();
  assert!(validate_link_targets(composite, &all_bkms).is_ok());
}

// --- L5: pricing output structure ---

#[test]
fn _0029() {
  // PricingOutput has all rate components
  let registry = make_loan_pricing_registry();
  let po = &registry.get("PricingOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(po, &["base rate"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(po, &["margin"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(po, &["all in rate"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(po, &["relationship discount"]), Some(FeelType::Number));
}

#[test]
fn _0030() {
  // LimitTermOutput has facility size and term constraints
  let registry = make_loan_pricing_registry();
  let lt = &registry.get("LimitTermOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(lt, &["max facility"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(lt, &["max term months"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(lt, &["ltv cap"]), Some(FeelType::Number));
}

// --- L6: final offer structure and audit trail ---

#[test]
fn _0031() {
  // LoanOfferOutput has top-level outcome and amount fields
  let registry = make_loan_pricing_registry();
  let lo = &registry.get("LoanOfferOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(lo, &["outcome"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(lo, &["offered rate"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(lo, &["offered amount"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(lo, &["offered term months"]), Some(FeelType::Number));
}

#[test]
fn _0032() {
  // LoanOfferOutput has nested conditions context
  let registry = make_loan_pricing_registry();
  let lo = &registry.get("LoanOfferOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(lo, &["conditions", "valuation required"]), Some(FeelType::Boolean));
  assert_eq!(resolve_field_chain(lo, &["conditions", "additional documentation"]), Some(FeelType::String));
}

#[test]
fn _0033() {
  // LoanOfferOutput audit trail carries the full explainability chain
  let registry = make_loan_pricing_registry();
  let lo = &registry.get("LoanOfferOutput").unwrap().feel_type;
  for field in [
    "credit band",
    "affordability band",
    "affordability reason",
    "ltv band",
    "composite risk",
    "policy exception",
    "income modifier applied",
  ] {
    assert_eq!(
      resolve_field_chain(lo, &["audit trail", field]),
      Some(FeelType::String),
      "audit trail missing field: {field}"
    );
  }
}

#[test]
fn _0034() {
  // LoanOfferOutput has optional decline_reason_code and conditions
  let registry = make_loan_pricing_registry();
  let entry = registry.get("LoanOfferOutput").unwrap();
  assert_eq!(entry.optional_fields.get("decline reason code"), Some(&true));
  assert_eq!(entry.optional_fields.get("conditions"), Some(&true));
}

#[test]
fn _0035() {
  // LoanOutcome enum validates the three possible outcomes
  let registry = make_loan_pricing_registry();
  let lo = registry.get("LoanOutcome").unwrap();
  assert!(validate_allowed_value("approve", lo).is_ok());
  assert!(validate_allowed_value("refer", lo).is_ok());
  assert!(validate_allowed_value("decline", lo).is_ok());
  assert!(validate_allowed_value("pending", lo).is_err());
}

// --- L6: DRD link validation (governed-by knowledge sources) ---

#[test]
fn _0036() {
  // Loan Offer governed-by three knowledge sources passes validation
  let nodes = make_loan_pricing_nodes();
  let offer = nodes.iter().find(|n| n.id == "loan_offer").unwrap();
  let knowledge_sources: Vec<DmnNode> = nodes.iter().filter(|n| n.node_type == "knowledge-source").cloned().collect();
  assert!(validate_link_targets(offer, &knowledge_sources).is_ok());
}

#[test]
fn _0037() {
  // Loan Offer governed-by a decision (wrong node type) fails validation
  let nodes = make_loan_pricing_nodes();
  let offer_node = nodes.iter().find(|n| n.id == "loan_offer").unwrap().clone();
  // Replace knowledge sources with a decision node that has a matching id
  let fake_decision = DmnNode {
    id: "credit_policy_framework".to_string(),
    node_type: "decision".to_string(),
    name: "Credit Policy Framework".to_string(),
    data_type: None,
    signature: None,
    uri: None,
    owner: None,
    requires: None,
    governed_by: None,
    supported_by: None,
    feel_expression: None,
    output_name: None,
  };
  let result = validate_link_targets(&offer_node, &[fake_decision]);
  assert!(result.is_err());
  let err_msg = result.unwrap_err().to_string();
  assert!(err_msg.contains("governed-by"));
  assert!(err_msg.contains("knowledge-source"));
}

#[test]
fn _0038() {
  // Policy Exceptions decision is governed-by Credit Policy Framework
  let nodes = make_loan_pricing_nodes();
  let pe = nodes.iter().find(|n| n.id == "policy_exceptions").unwrap();
  let knowledge_sources: Vec<DmnNode> = nodes.iter().filter(|n| n.node_type == "knowledge-source").cloned().collect();
  assert!(validate_link_targets(pe, &knowledge_sources).is_ok());
}

// --- Cross-layer: every decision's output type resolves ---

#[test]
fn _0039() {
  // Every decision and input-data node's data-type resolves through the registry
  let registry = make_loan_pricing_registry();
  let nodes = make_loan_pricing_nodes();
  for node in nodes.iter().filter(|n| n.data_type.is_some()) {
    let dt = node.data_type.as_ref().unwrap();
    let result = resolve_data_type(dt, Path::new("."), &registry);
    assert!(result.is_ok(), "node '{}' (type '{}') failed to resolve: {}", node.name, dt.type_ref, result.unwrap_err());
  }
}

// --- Cross-layer: DRD chaining depth (L3 → L4 → L5 → L6) ---

#[test]
fn _0040() {
  // Verify the full requires chain: L6 → L5 → L4 → L3
  let nodes = make_loan_pricing_nodes();
  let find = |id: &str| nodes.iter().find(|n| n.id == id).unwrap();

  // L6 requires L5 outputs and L4 policy exceptions
  let offer = find("loan_offer");
  let offer_requires = offer.requires.as_ref().unwrap();
  assert!(offer_requires.contains(&"pricing_margin".to_string()));
  assert!(offer_requires.contains(&"limit_and_term".to_string()));
  assert!(offer_requires.contains(&"policy_exceptions".to_string()));

  // L5 pricing requires L4 composite risk + L2 classifiers
  let pricing = find("pricing_margin");
  let pricing_requires = pricing.requires.as_ref().unwrap();
  assert!(pricing_requires.contains(&"composite_risk".to_string()));
  assert!(pricing_requires.contains(&"product_type".to_string()));
  assert!(pricing_requires.contains(&"jurisdiction".to_string()));
  assert!(pricing_requires.contains(&"relationship_tier".to_string()));

  // L5 limit requires L4 composite + L3 collateral LTV
  let limit = find("limit_and_term");
  let limit_requires = limit.requires.as_ref().unwrap();
  assert!(limit_requires.contains(&"composite_risk".to_string()));
  assert!(limit_requires.contains(&"collateral_ltv".to_string()));

  // L4 composite requires all three L3 sub-scores
  let composite = find("composite_risk");
  let composite_requires = composite.requires.as_ref().unwrap();
  assert_eq!(composite_requires.len(), 3);
}

// --- Registry merge: adding a new product line ---

#[test]
fn _0041() {
  // Merging a trade-finance registry into the loan-pricing registry
  let mut registry = make_loan_pricing_registry();
  let original_count = registry.len();

  let mut trade_registry = TypeRegistry::new();
  trade_registry
    .insert(enum_entry("TradeInstrument", &["letter of credit", "bank guarantee", "documentary collection"]))
    .unwrap();
  trade_registry
    .insert(simple_entry(
      "TradeFinanceOutput",
      ctx(&[
        ("instrument", FeelType::String),
        ("facility limit", FeelType::Number),
        ("country risk band", FeelType::String),
      ]),
    ))
    .unwrap();

  registry.merge(trade_registry).unwrap();
  assert_eq!(registry.len(), original_count + 2);

  let ti = registry.get("TradeInstrument").unwrap();
  assert!(validate_allowed_value("letter of credit", ti).is_ok());
  assert!(validate_allowed_value("swap", ti).is_err());

  let tfo = &registry.get("TradeFinanceOutput").unwrap().feel_type;
  assert_eq!(resolve_field_chain(tfo, &["facility limit"]), Some(FeelType::Number));
}

// --- Registry conflict: duplicate type from different source ---

#[test]
fn _0042() {
  // Inserting a conflicting type from a different source file is rejected
  let mut registry = make_loan_pricing_registry();
  let conflicting = TypeEntry {
    name: "ProductType".to_string(),
    feel_type: FeelType::String,
    source: TypeSource::JsonSchema(std::path::PathBuf::from("other-schema.json")),
    allowed_values: Some(vec!["mortgage".to_string()]),
    optional_fields: HashMap::new(),
  };
  let result = registry.insert(conflicting);
  assert!(result.is_err());
  assert!(result.unwrap_err().to_string().contains("ambiguous"));
}

// --- End-to-end: full pipeline type resolution ---

#[test]
fn _0043() {
  // End-to-end: resolve types at every DRD layer, verify field chains,
  // validate enum constraints, and confirm audit trail completeness.
  let registry = make_loan_pricing_registry();

  // L1 → resolve raw input type, drill into fields
  let app_ref = DataTypeRef {
    type_ref: "LoanApplication".to_string(),
    schema: None,
  };
  let app = resolve_data_type(&app_ref, Path::new("."), &registry).unwrap();
  assert_eq!(resolve_field_chain(&app.feel_type, &["gross annual income"]), Some(FeelType::Number));

  // L2 → validate classifier enums
  let product = registry.get("ProductType").unwrap();
  assert!(validate_allowed_value("mortgage", product).is_ok());
  assert!(validate_allowed_value("derivative", product).is_err());

  // L3 → resolve BKM output type, verify sub-score structure
  let afford_ref = DataTypeRef {
    type_ref: "AffordabilityOutput".to_string(),
    schema: None,
  };
  let afford = resolve_data_type(&afford_ref, Path::new("."), &registry).unwrap();
  assert_eq!(resolve_field_chain(&afford.feel_type, &["dti ratio"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(&afford.feel_type, &["reason code"]), Some(FeelType::String));

  // L3 → validate affordability band value
  let aband = registry.get("AffordabilityBand").unwrap();
  assert!(validate_allowed_value("Refer", aband).is_ok());

  // L4 → resolve composite risk, drill into nested sub-score summary
  let comp_ref = DataTypeRef {
    type_ref: "CompositeRiskOutput".to_string(),
    schema: None,
  };
  let comp = resolve_data_type(&comp_ref, Path::new("."), &registry).unwrap();
  assert_eq!(resolve_field_chain(&comp.feel_type, &["sub score summary", "affordability"]), Some(FeelType::String));

  // L5 → resolve pricing output
  let price_ref = DataTypeRef {
    type_ref: "PricingOutput".to_string(),
    schema: None,
  };
  let price = resolve_data_type(&price_ref, Path::new("."), &registry).unwrap();
  assert_eq!(resolve_field_chain(&price.feel_type, &["all in rate"]), Some(FeelType::Number));
  assert_eq!(resolve_field_chain(&price.feel_type, &["relationship discount"]), Some(FeelType::Number));

  // L6 → resolve final offer, verify audit trail carries the full explainability chain
  let offer_ref = DataTypeRef {
    type_ref: "LoanOfferOutput".to_string(),
    schema: None,
  };
  let offer = resolve_data_type(&offer_ref, Path::new("."), &registry).unwrap();
  assert_eq!(resolve_field_chain(&offer.feel_type, &["outcome"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(&offer.feel_type, &["audit trail", "credit band"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(&offer.feel_type, &["audit trail", "affordability band"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(&offer.feel_type, &["audit trail", "affordability reason"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(&offer.feel_type, &["audit trail", "income modifier applied"]), Some(FeelType::String));
  assert_eq!(resolve_field_chain(&offer.feel_type, &["audit trail", "policy exception"]), Some(FeelType::String));

  // Validate outcome enum
  let outcome = registry.get("LoanOutcome").unwrap();
  assert!(validate_allowed_value("decline", outcome).is_ok());
}

// --- End-to-end: DRD link validation across the full graph ---

#[test]
fn _0044() {
  // Every decision node's governed-by and supported-by links resolve to correct node types
  let nodes = make_loan_pricing_nodes();
  for node in nodes.iter().filter(|n| n.node_type == "decision") {
    let result = validate_link_targets(node, &nodes);
    assert!(result.is_ok(), "link validation failed for decision '{}': {}", node.name, result.unwrap_err());
  }
}

// --- End-to-end: all BKM signatures valid ---

#[test]
fn _0045() {
  // Every BKM node in the DRD passes signature validation
  let nodes = make_loan_pricing_nodes();
  for node in nodes.iter().filter(|n| n.node_type == "bkm") {
    let result = validate_bkm_signature(node);
    assert!(result.is_ok(), "BKM '{}' failed signature validation: {}", node.name, result.unwrap_err());
  }
}

// --- Explainability: audit trail field completeness ---

#[test]
fn _0046() {
  // The audit trail in LoanOfferOutput must carry exactly the fields
  // required to answer "why was this application declined?" per
  // FCA Consumer Duty / ECOA requirements.
  let registry = make_loan_pricing_registry();
  let offer = &registry.get("LoanOfferOutput").unwrap().feel_type;

  // These are the mandatory audit fields for regulatory compliance:
  let required_audit_fields = [
    "credit band",             // from L3 CreditBanding
    "affordability band",      // from L3 Affordability (Table B output)
    "affordability reason",    // from L3 Affordability (reason code, e.g. AFC-07)
    "ltv band",                // from L3 CollateralLTV
    "composite risk",          // from L4 CompositeRiskRating
    "policy exception",        // from L4 PolicyExceptions (exception code if triggered)
    "income modifier applied", // from L3 Income Verification Modifier (e.g. INC-02)
  ];

  for field in required_audit_fields {
    let resolved = resolve_field_chain(offer, &["audit trail", field]);
    assert!(resolved.is_some(), "audit trail missing mandatory field: '{field}'");
    assert_eq!(resolved, Some(FeelType::String), "audit trail field '{field}' should be String");
  }

  // Negative: demographic data must NOT appear in the audit trail
  assert!(resolve_field_chain(offer, &["audit trail", "gender"]).is_none());
  assert!(resolve_field_chain(offer, &["audit trail", "ethnicity"]).is_none());
  assert!(resolve_field_chain(offer, &["audit trail", "age"]).is_none());
}
