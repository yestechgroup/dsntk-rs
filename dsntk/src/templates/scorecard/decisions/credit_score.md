---
dmn:
  id: credit_score
  type: decision
  name: Credit Score
  data-type:
    ref: ScoreBreakdown
    schema: ../types/scorecard.ts
  requires:
    - credit_input
    - payment_score
    - debt_score
    - account_score
---

# Credit Score

Computes a weighted composite credit score from three sub-scores.

## Weights

| Component      | Weight |
|---------------|--------|
| Payment score | 40%    |
| Debt score    | 35%    |
| Account score | 25%    |

## Formula

```
compositeScore = paymentScore * 0.40 + debtScore * 0.35 + accountScore * 0.25
```
