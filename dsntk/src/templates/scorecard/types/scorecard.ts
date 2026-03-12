// Credit scorecard type definitions.

export type PaymentHistory = "Excellent" | "Good" | "Fair" | "Poor";

export interface CreditInput {
  paymentHistory: PaymentHistory;
  debtRatio: number;
  accountAgeYears: number;
}

export interface ScoreBreakdown {
  paymentScore: number;
  debtScore: number;
  accountScore: number;
  compositeScore: number;
}
