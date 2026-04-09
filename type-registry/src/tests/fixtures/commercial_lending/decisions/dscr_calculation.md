---
dmn:
  id: dscr_calculation
  type: bkm
  name: DSCR Calculation
  feel-expression: (Net Profit + Depreciation + Interest Expense) / Annual Debt Service
  output-name: DSCR
  requires:
    - financial_statements
    - loan_request
---

# DSCR Calculation

Computes Debt Service Coverage Ratio using EBITDA proxy:

```
EBITDA = netProfit + depreciation + interestExpense
DSCR = EBITDA / annualDebtService
```

Where `annualDebtService` includes the proposed new facility's annual
repayment obligation plus existing debt service.

A DSCR of 1.0 means the business generates exactly enough cash to cover
its debt obligations. The policy minimum is 1.0 (hard stop) but comfortable
lending typically requires 1.25 or above.
