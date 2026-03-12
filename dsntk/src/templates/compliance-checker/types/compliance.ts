// Compliance checker type definitions.

export interface ProductInput {
  productName: string;
  certifications: string[];
  targetCountry: string;
}

export type ComplianceStatus = "Compliant" | "Non-Compliant" | "Review Required";

export interface ComplianceResult {
  status: ComplianceStatus;
  hasCertifications: boolean;
  countryAllowed: boolean;
  reason: string;
}
