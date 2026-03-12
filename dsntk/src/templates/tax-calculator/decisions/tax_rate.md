---
dmn:
  id: tax_rate
  type: decision
  name: Tax Rate
  data-type:
    ref: number
  requires:
    - taxpayer_input
---

# Tax Rate

Determines the applicable tax rate based on income bracket and filing status.
Uses **UNIQUE** hit policy.

## Decision table

| Rule | Annual income       | Filing status | Tax rate |
|------|---------------------|---------------|----------|
| T1   | <= 10000            | Single        | 0.10     |
| T2   | (10000, 40000]      | Single        | 0.20     |
| T3   | > 40000             | Single        | 0.30     |
| T4   | <= 20000            | Married       | 0.10     |
| T5   | (20000, 80000]      | Married       | 0.20     |
| T6   | > 80000             | Married       | 0.30     |
