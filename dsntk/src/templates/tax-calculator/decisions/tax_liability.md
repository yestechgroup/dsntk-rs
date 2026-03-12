---
dmn:
  id: tax_liability
  type: decision
  name: Tax Liability
  data-type:
    ref: TaxResult
    schema: ../types/tax.ts
  requires:
    - taxpayer_input
    - tax_rate
---

# Tax Liability

Calculates the final tax amount by applying the tax rate to annual income.

## Formula

```
taxAmount = annualIncome * taxRate
```
