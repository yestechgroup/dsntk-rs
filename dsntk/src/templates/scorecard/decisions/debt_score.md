---
dmn:
  id: debt_score
  type: bkm
  name: Debt Score
  data-type:
    ref: number
  signature:
    parameters:
      - name: debtRatio
        type: number
    return-type: number
---

# Debt Score

Scores debt-to-income ratio on a 0-100 scale.

## Decision table

| Rule | Debt ratio   | Score |
|------|-------------|-------|
| D1   | <= 0.2      | 100   |
| D2   | (0.2, 0.4]  | 70    |
| D3   | (0.4, 0.6]  | 40    |
| D4   | > 0.6       | 10    |
