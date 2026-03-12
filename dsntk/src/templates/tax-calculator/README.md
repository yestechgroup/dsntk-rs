# Tax Calculator

A Markdown-native DMN project that calculates tax liability using progressive brackets.

## Showcases

- Decision tables with numeric ranges
- Multi-input matching on income and filing status
- Chained calculation (rate lookup then multiplication)

## Project structure

```
tax-calculator/
├── README.md
├── decisions/
│   ├── taxpayer_input.md          # input-data: income and filing status
│   ├── tax_rate.md                # decision: bracket-based rate lookup
│   └── tax_liability.md           # decision: final tax amount
└── types/
    └── tax.ts                     # TypeScript type definitions
```

## How to run

```bash
dsntk typ tax-calculator/
```
