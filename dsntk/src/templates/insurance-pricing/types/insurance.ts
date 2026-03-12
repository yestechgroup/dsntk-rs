// Insurance pricing type definitions.

export type CoverageLevel = "Basic" | "Standard" | "Premium";

export interface PolicyInput {
  applicantAge: number;
  hasPriorClaims: boolean;
  coverageLevel: CoverageLevel;
}

export interface PremiumResult {
  basePremium: number;
  ageFactor: number;
  claimsSurcharge: number;
  totalPremium: number;
}
