---
dmn:
  id: eligibility
  type: decision
  name: Eligibility
  requires:
    - applicant
    - loan_amount
---

| U  | Age | Loan Amount | Eligible |
|:--:|:---:|:-----------:|:--------:|
|    | in  | in          | out      |
| 1  | >=18| <=500000    | true     |
| 2  | -   | -           | false    |
