// Order routing type definitions.

export type Region = "Domestic" | "International";

export type Priority = "Standard" | "Express" | "Overnight";

export interface OrderInput {
  weightKg: number;
  destinationRegion: Region;
  priority: Priority;
}

export interface ShippingResult {
  cost: number;
  fulfillmentCenter: string;
}
