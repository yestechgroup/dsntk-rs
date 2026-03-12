---
dmn:
  id: account_score
  type: bkm
  name: Account Score
  data-type:
    ref: number
  signature:
    parameters:
      - name: accountAgeYears
        type: number
    return-type: number
---

# Account Score

Scores account age on a 0-100 scale.

## Decision table

| Rule | Account age (years) | Score |
|------|---------------------|-------|
| A1   | >= 10               | 100   |
| A2   | [5, 10)             | 70    |
| A3   | [2, 5)              | 40    |
| A4   | < 2                 | 20    |
