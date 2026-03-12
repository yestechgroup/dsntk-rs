---
dmn:
  id: compliance_result
  type: decision
  name: Compliance Result
  data-type:
    ref: ComplianceResult
    schema: ../types/compliance.ts
  requires:
    - product_input
    - certification_check
  governed-by:
    - regulatory_standard
---

# Compliance Result

Assembles the final compliance determination from certification check
and country validation.

## Decision table

| Rule | Certification check | Country restricted | Product name valid | Status         | Reason                     |
|------|--------------------|--------------------|-------------------|----------------|----------------------------|
| C1   | true               | false              | true              | Compliant      | All checks passed          |
| C2   | false              | Any                | Any               | Non-Compliant  | Missing certifications     |
| C3   | Any                | true               | Any               | Non-Compliant  | Restricted country         |
| C4   | true               | false              | false             | Review Required | Product name missing       |
