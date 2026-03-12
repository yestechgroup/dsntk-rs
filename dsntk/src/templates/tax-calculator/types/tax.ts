// Tax calculator type definitions.

export type FilingStatus = "Single" | "Married";

export interface TaxpayerInput {
  annualIncome: number;
  filingStatus: FilingStatus;
}

export interface TaxResult {
  taxRate: number;
  taxAmount: number;
}
