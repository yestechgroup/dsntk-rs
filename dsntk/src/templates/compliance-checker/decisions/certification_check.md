---
dmn:
  id: certification_check
  type: decision
  name: Certification Check
  data-type:
    ref: boolean
  requires:
    - product_input
  governed-by:
    - regulatory_standard
---

# Certification Check

Validates that the product holds all required certifications (ISO 9001 and CE).

## Logic

```
every cert in ["ISO9001", "CE"] satisfies list contains(certifications, cert)
```
