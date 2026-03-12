# Scorecard

A Markdown-native DMN project implementing a credit scoring pattern with weighted factors.

## Showcases

- Multiple BKMs feeding into a composite decision
- Weighted scoring with literal expression
- Chained decision contexts across multiple sub-scores

## Project structure

```
scorecard/
├── README.md
├── decisions/
│   ├── credit_input.md            # input-data: credit profile
│   ├── payment_score.md           # bkm: payment history scoring
│   ├── debt_score.md              # bkm: debt ratio scoring
│   ├── account_score.md           # bkm: account age scoring
│   └── credit_score.md            # decision: weighted composite score
└── types/
    └── scorecard.ts               # TypeScript type definitions
```

## How to run

```bash
dsntk typ scorecard/
```
