# Loan Eligibility

A Markdown-native DMN project that determines loan eligibility based on applicant data.

## Showcases

- Chained decisions (eligibility depends on risk rating)
- UNIQUE hit policy for risk classification
- TypeScript type definitions for inputs and outputs
- Markdown decision tables with FEEL expressions

## Project structure

```
loan-eligibility/
├── README.md
├── decisions/
│   ├── applicant_input.md        # input-data: applicant details
│   ├── risk_rating.md            # decision: credit risk classification
│   └── loan_eligibility.md       # decision: final approval/decline
└── types/
    └── loan.ts                   # TypeScript type definitions
```

## How to run

```bash
# Validate types
dsntk typ loan-eligibility/

# (future) Evaluate the decision model
dsntk edm -i "Loan Eligibility" loan-eligibility/
```
