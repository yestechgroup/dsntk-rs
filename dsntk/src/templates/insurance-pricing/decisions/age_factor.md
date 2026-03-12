---
dmn:
  id: age_factor
  type: bkm
  name: Age Factor
  data-type:
    ref: number
  signature:
    parameters:
      - name: applicantAge
        type: number
    return-type: number
---

# Age Factor

Determines a pricing multiplier based on the applicant's age range.

## Decision table

| Rule | Applicant age | Age factor |
|------|---------------|------------|
| A1   | [18, 25]      | 1.5        |
| A2   | [26, 45]      | 1.0        |
| A3   | [46, 65]      | 1.3        |
| A4   | > 65          | 1.7        |
