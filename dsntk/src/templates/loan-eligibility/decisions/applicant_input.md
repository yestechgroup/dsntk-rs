---
dmn:
  id: applicant_input
  type: input-data
  name: Applicant Data
  data-type:
    ref: ApplicantData
    schema: ../types/loan.ts
---

# Applicant Data

Raw application data submitted by the loan applicant.

Includes age, annual income, and credit score from bureau lookup.
