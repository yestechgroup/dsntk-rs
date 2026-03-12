---
dmn:
  id: fulfillment_center
  type: decision
  name: Fulfillment Center
  data-type:
    ref: string
  requires:
    - order_input
---

# Fulfillment Center

Routes orders to the appropriate fulfillment center based on region and weight.
Uses **UNIQUE** hit policy.

## Decision table

| Rule | Region        | Weight (kg) | Fulfillment center     |
|------|---------------|-------------|------------------------|
| F1   | Domestic      | <= 5        | Local Warehouse        |
| F2   | Domestic      | > 5         | Regional Hub           |
| F3   | International | Any         | International Gateway  |
