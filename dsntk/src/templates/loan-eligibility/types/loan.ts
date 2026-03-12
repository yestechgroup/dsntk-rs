// Loan eligibility type definitions.

export interface ApplicantData {
  age: number;
  annualIncome: number;
  creditScore: number;
}

export type RiskLevel = "Low" | "Medium" | "High";

export interface EligibilityResult {
  decision: string;
  riskRating: RiskLevel;
  reason: string;
}
