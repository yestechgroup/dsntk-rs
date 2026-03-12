//! # Built-in project templates for Markdown-native DMN

/// A single file within a template.
pub struct TemplateFile {
  /// Relative path of the file within the scaffolded project.
  pub path: &'static str,
  /// Content of the file.
  pub content: &'static str,
}

/// A project template that can be scaffolded.
pub struct Template {
  /// Short name used on the command line (e.g. "loan-eligibility").
  pub name: &'static str,
  /// Human-readable description shown in `--list` output.
  pub description: &'static str,
  /// Files that make up this template.
  pub files: &'static [TemplateFile],
}

// --- loan-eligibility ---

const LOAN_ELIGIBILITY: Template = Template {
  name: "loan-eligibility",
  description: "Chained decisions with UNIQUE hit policy for loan approval",
  files: &[
    TemplateFile {
      path: "README.md",
      content: include_str!("loan-eligibility/README.md"),
    },
    TemplateFile {
      path: "types/loan.ts",
      content: include_str!("loan-eligibility/types/loan.ts"),
    },
    TemplateFile {
      path: "decisions/applicant_input.md",
      content: include_str!("loan-eligibility/decisions/applicant_input.md"),
    },
    TemplateFile {
      path: "decisions/risk_rating.md",
      content: include_str!("loan-eligibility/decisions/risk_rating.md"),
    },
    TemplateFile {
      path: "decisions/loan_eligibility.md",
      content: include_str!("loan-eligibility/decisions/loan_eligibility.md"),
    },
  ],
};

// --- insurance-pricing ---

const INSURANCE_PRICING: Template = Template {
  name: "insurance-pricing",
  description: "BKM with age-based pricing, FEEL ranges, and literal expressions",
  files: &[
    TemplateFile {
      path: "README.md",
      content: include_str!("insurance-pricing/README.md"),
    },
    TemplateFile {
      path: "types/insurance.ts",
      content: include_str!("insurance-pricing/types/insurance.ts"),
    },
    TemplateFile {
      path: "decisions/policy_input.md",
      content: include_str!("insurance-pricing/decisions/policy_input.md"),
    },
    TemplateFile {
      path: "decisions/age_factor.md",
      content: include_str!("insurance-pricing/decisions/age_factor.md"),
    },
    TemplateFile {
      path: "decisions/premium_calculation.md",
      content: include_str!("insurance-pricing/decisions/premium_calculation.md"),
    },
  ],
};

// --- tax-calculator ---

const TAX_CALCULATOR: Template = Template {
  name: "tax-calculator",
  description: "Progressive tax brackets with numeric ranges and chained calculation",
  files: &[
    TemplateFile {
      path: "README.md",
      content: include_str!("tax-calculator/README.md"),
    },
    TemplateFile {
      path: "types/tax.ts",
      content: include_str!("tax-calculator/types/tax.ts"),
    },
    TemplateFile {
      path: "decisions/taxpayer_input.md",
      content: include_str!("tax-calculator/decisions/taxpayer_input.md"),
    },
    TemplateFile {
      path: "decisions/tax_rate.md",
      content: include_str!("tax-calculator/decisions/tax_rate.md"),
    },
    TemplateFile {
      path: "decisions/tax_liability.md",
      content: include_str!("tax-calculator/decisions/tax_liability.md"),
    },
  ],
};

// --- order-routing ---

const ORDER_ROUTING: Template = Template {
  name: "order-routing",
  description: "Multi-input decision tables for logistics branching",
  files: &[
    TemplateFile {
      path: "README.md",
      content: include_str!("order-routing/README.md"),
    },
    TemplateFile {
      path: "types/order.ts",
      content: include_str!("order-routing/types/order.ts"),
    },
    TemplateFile {
      path: "decisions/order_input.md",
      content: include_str!("order-routing/decisions/order_input.md"),
    },
    TemplateFile {
      path: "decisions/shipping_cost.md",
      content: include_str!("order-routing/decisions/shipping_cost.md"),
    },
    TemplateFile {
      path: "decisions/fulfillment_center.md",
      content: include_str!("order-routing/decisions/fulfillment_center.md"),
    },
  ],
};

// --- compliance-checker ---

const COMPLIANCE_CHECKER: Template = Template {
  name: "compliance-checker",
  description: "Knowledge sources, governed-by relationships, and boolean logic",
  files: &[
    TemplateFile {
      path: "README.md",
      content: include_str!("compliance-checker/README.md"),
    },
    TemplateFile {
      path: "types/compliance.ts",
      content: include_str!("compliance-checker/types/compliance.ts"),
    },
    TemplateFile {
      path: "decisions/product_input.md",
      content: include_str!("compliance-checker/decisions/product_input.md"),
    },
    TemplateFile {
      path: "decisions/regulatory_standard.md",
      content: include_str!("compliance-checker/decisions/regulatory_standard.md"),
    },
    TemplateFile {
      path: "decisions/certification_check.md",
      content: include_str!("compliance-checker/decisions/certification_check.md"),
    },
    TemplateFile {
      path: "decisions/compliance_result.md",
      content: include_str!("compliance-checker/decisions/compliance_result.md"),
    },
  ],
};

// --- scorecard ---

const SCORECARD: Template = Template {
  name: "scorecard",
  description: "Weighted BKM scoring with chained decision contexts",
  files: &[
    TemplateFile {
      path: "README.md",
      content: include_str!("scorecard/README.md"),
    },
    TemplateFile {
      path: "types/scorecard.ts",
      content: include_str!("scorecard/types/scorecard.ts"),
    },
    TemplateFile {
      path: "decisions/credit_input.md",
      content: include_str!("scorecard/decisions/credit_input.md"),
    },
    TemplateFile {
      path: "decisions/payment_score.md",
      content: include_str!("scorecard/decisions/payment_score.md"),
    },
    TemplateFile {
      path: "decisions/debt_score.md",
      content: include_str!("scorecard/decisions/debt_score.md"),
    },
    TemplateFile {
      path: "decisions/account_score.md",
      content: include_str!("scorecard/decisions/account_score.md"),
    },
    TemplateFile {
      path: "decisions/credit_score.md",
      content: include_str!("scorecard/decisions/credit_score.md"),
    },
  ],
};

// --- registry ---

/// All available templates.
const ALL_TEMPLATES: &[Template] = &[LOAN_ELIGIBILITY, INSURANCE_PRICING, TAX_CALCULATOR, ORDER_ROUTING, COMPLIANCE_CHECKER, SCORECARD];

/// Returns all available templates.
pub fn all_templates() -> &'static [Template] {
  ALL_TEMPLATES
}

/// Looks up a template by name.
pub fn get_template(name: &str) -> Option<&'static Template> {
  ALL_TEMPLATES.iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::collections::HashSet;

  #[test]
  fn _0001() {
    let names: Vec<_> = all_templates().iter().map(|t| t.name).collect();
    let unique: HashSet<_> = names.iter().collect();
    assert_eq!(names.len(), unique.len(), "duplicate template names found");
  }

  #[test]
  fn _0002() {
    for t in all_templates() {
      assert!(!t.description.is_empty(), "template '{}' has no description", t.name);
    }
  }

  #[test]
  fn _0003() {
    for t in all_templates() {
      assert!(!t.files.is_empty(), "template '{}' has no files", t.name);
    }
  }

  #[test]
  fn _0004() {
    for t in all_templates() {
      let has_readme = t.files.iter().any(|f| f.path == "README.md");
      assert!(has_readme, "template '{}' is missing README.md", t.name);
    }
  }

  #[test]
  fn _0005() {
    for t in all_templates() {
      let has_decision_file = t.files.iter().any(|f| f.path.starts_with("decisions/") && f.path.ends_with(".md"));
      assert!(has_decision_file, "template '{}' has no decision .md file", t.name);
    }
  }

  #[test]
  fn _0006() {
    let t = get_template("loan-eligibility").expect("loan-eligibility template not found");
    assert_eq!(t.name, "loan-eligibility");
  }

  #[test]
  fn _0007() {
    assert!(get_template("does-not-exist").is_none());
  }

  #[test]
  fn _0008() {
    let expected = [
      "loan-eligibility",
      "insurance-pricing",
      "tax-calculator",
      "order-routing",
      "compliance-checker",
      "scorecard",
    ];
    for name in expected {
      assert!(get_template(name).is_some(), "template '{}' missing from registry", name);
    }
  }

  #[test]
  fn _0009() {
    for t in all_templates() {
      let has_types = t.files.iter().any(|f| f.path.starts_with("types/") && f.path.ends_with(".ts"));
      assert!(has_types, "template '{}' has no TypeScript type file", t.name);
    }
  }

  #[test]
  fn _0010() {
    for t in all_templates() {
      let has_input_data = t.files.iter().any(|f| f.path.starts_with("decisions/") && f.content.contains("type: input-data"));
      assert!(has_input_data, "template '{}' has no input-data node", t.name);
    }
  }

  #[test]
  fn _0011() {
    for t in all_templates() {
      for f in t.files {
        if f.path.starts_with("decisions/") && f.path.ends_with(".md") {
          assert!(f.content.contains("---"), "decision file '{}' in template '{}' has no YAML front matter", f.path, t.name);
          assert!(
            f.content.contains("dmn:"),
            "decision file '{}' in template '{}' has no dmn: key in front matter",
            f.path,
            t.name
          );
        }
      }
    }
  }
}
