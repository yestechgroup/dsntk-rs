---
dmn:
  id: loan_eligibility
  type: decision
  name: Loan Eligibility
  data-type:
    ref: EligibilityResult
    schema: ../types/loan.ts
  requires:
    - applicant_input
    - risk_rating
---

# Loan Eligibility

Determines whether the loan application is approved or declined based on
age, income, and risk rating from upstream decisions.

Uses **UNIQUE** hit policy with priority ordering.

## Decision table

| Rule | Age     | Annual income | Risk rating | Decision  | Reason                    |
|------|---------|---------------|-------------|-----------|---------------------------|
| E1   | [18,70] | >= 30000      | Low         | Approved  | Low risk, sufficient income |
| E2   | [18,70] | >= 50000      | Medium      | Approved  | Medium risk, high income    |
| E3   | Any     | Any           | High        | Declined  | High credit risk            |
| E4   | < 18    | Any           | Any         | Declined  | Applicant under 18          |
| E5   | > 70    | Any           | Any         | Declined  | Applicant over 70           |
| E6   | [18,70] | < 30000       | Low         | Declined  | Insufficient income         |
| E7   | [18,70] | < 50000       | Medium      | Declined  | Insufficient income for risk |
