# Order Routing

A Markdown-native DMN project that routes orders to fulfillment centers.

## Showcases

- Multi-input decision tables
- Multiple independent decisions from shared inputs
- Logistics branching based on region and weight

## Project structure

```
order-routing/
├── README.md
├── decisions/
│   ├── order_input.md             # input-data: order details
│   ├── shipping_cost.md           # decision: cost calculation
│   └── fulfillment_center.md      # decision: routing destination
└── types/
    └── order.ts                   # TypeScript type definitions
```

## How to run

```bash
dsntk typ order-routing/
```
