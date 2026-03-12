---
dmn:
  id: premium_calculation
  type: decision
  name: Premium Calculation
  data-type:
    ref: PremiumResult
    schema: ../types/insurance.ts
  requires:
    - policy_input
    - age_factor
---

# Premium Calculation

Calculates the total insurance premium by combining:

1. **Base premium** derived from coverage level
2. **Age factor** from the Age Factor BKM
3. **Claims surcharge** for applicants with prior claims

## Base premium by coverage level

| Coverage level | Base premium |
|---------------|-------------|
| Basic         | 500         |
| Standard      | 1000        |
| Premium       | 2000        |

## Formula

```
totalPremium = basePremium * ageFactor * (1 + claimsSurcharge)
```

where `claimsSurcharge` is 0.4 if `hasPriorClaims` is true, otherwise 0.
