---
dmn:
  id: risk_rating
  type: decision
  name: Risk Rating
  data-type:
    ref: RiskLevel
    schema: ../types/loan.ts
  requires:
    - applicant_input
---

# Risk Rating

Classifies the applicant's credit risk based on credit score.
Uses **UNIQUE** hit policy — exactly one rule must fire.

## Decision table

| Rule | Credit score | Risk rating |
|------|-------------|-------------|
| R1   | >= 720      | Low         |
| R2   | [620, 720)  | Medium      |
| R3   | < 620       | High        |
