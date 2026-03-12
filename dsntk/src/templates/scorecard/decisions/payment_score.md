---
dmn:
  id: payment_score
  type: bkm
  name: Payment Score
  data-type:
    ref: number
  signature:
    parameters:
      - name: paymentHistory
        type: PaymentHistory
    return-type: number
    schema: ../types/scorecard.ts
---

# Payment Score

Scores payment history on a 0-100 scale.

## Decision table

| Rule | Payment history | Score |
|------|----------------|-------|
| P1   | Excellent      | 100   |
| P2   | Good           | 80    |
| P3   | Fair           | 50    |
| P4   | Poor           | 20    |
