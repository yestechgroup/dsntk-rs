---
dmn:
  id: shipping_cost
  type: decision
  name: Shipping Cost
  data-type:
    ref: number
  requires:
    - order_input
---

# Shipping Cost

Calculates shipping cost based on weight, region, and priority.
Uses **UNIQUE** hit policy.

## Decision table

| Rule | Weight (kg) | Region        | Priority  | Cost |
|------|-------------|---------------|-----------|------|
| S1   | <= 5        | Domestic      | Standard  | 5    |
| S2   | <= 5        | Domestic      | Express   | 15   |
| S3   | <= 5        | Domestic      | Overnight | 30   |
| S4   | > 5         | Domestic      | Standard  | 10   |
| S5   | > 5         | Domestic      | Express   | 25   |
| S6   | > 5         | Domestic      | Overnight | 50   |
| S7   | <= 5        | International | Standard  | 20   |
| S8   | <= 5        | International | Express   | 45   |
| S9   | <= 5        | International | Overnight | 80   |
| S10  | > 5         | International | Standard  | 35   |
| S11  | > 5         | International | Express   | 70   |
| S12  | > 5         | International | Overnight | 120  |
