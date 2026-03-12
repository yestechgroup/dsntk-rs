# Compliance Checker

A Markdown-native DMN project that checks regulatory compliance of a product.

## Showcases

- Knowledge source nodes for regulatory authority
- Boolean logic in decision tables
- `governed-by` relationship linking decisions to policy

## Project structure

```
compliance-checker/
├── README.md
├── decisions/
│   ├── product_input.md           # input-data: product details
│   ├── certification_check.md     # decision: certification validation
│   ├── compliance_result.md       # decision: overall compliance
│   └── regulatory_standard.md     # knowledge-source: authority reference
└── types/
    └── compliance.ts              # TypeScript type definitions
```

## How to run

```bash
dsntk typ compliance-checker/
```
