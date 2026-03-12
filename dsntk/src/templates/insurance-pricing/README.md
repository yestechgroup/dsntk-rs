# Insurance Pricing

A Markdown-native DMN project that calculates insurance premiums.

## Showcases

- BKM (Business Knowledge Model) with formal function signature
- Numeric FEEL ranges for age-based pricing
- Chained decision with literal expression

## Project structure

```
insurance-pricing/
├── README.md
├── decisions/
│   ├── policy_input.md           # input-data: policy details
│   ├── age_factor.md             # bkm: age-based multiplier
│   └── premium_calculation.md    # decision: final premium
└── types/
    └── insurance.ts              # TypeScript type definitions
```

## How to run

```bash
dsntk typ insurance-pricing/
```
