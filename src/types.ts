export interface Labels {
  appTitle: string;
  importButton: string;
  exportButton: string;
  shippingName: string;
  white: string;
  medium: string;
  dark: string;
  espresso: string;
  decaf: string;
  other: string;
  bagTotals: string;
  totalBags: string;
  rawAmounts: string;
  rawWhite: string;
  rawMedium: string;
  rawDark: string;
  rawDecaf: string;
  rawTotal: string;
}

export interface AppConfig {
  labels: Labels;
  productMappings: Record<string, string>;
}

export interface ConfigResponse {
  config: AppConfig;
  configPath: string;
  warning: string | null;
}

export interface RoastTotals {
  white: number;
  medium: number;
  dark: number;
  espresso: number;
  decaf: number;
  other: number;
}

export interface CustomerSummary {
  shippingName: string;
  totals: RoastTotals;
}

export interface ProcessedData {
  sourcePath: string;
  customers: CustomerSummary[];
  totals: RoastTotals;
  unmatchedProducts: string[];
  warnings: string[];
  lineItemCount: number;
}

export interface Factors {
  white: number;
  medium: number;
  dark: number;
  decaf: number;
}

export interface ExportSettings {
  factors: Factors;
  roundingIncrement: number;
  summaryPosition: "header" | "footer";
}

export interface RawAmounts {
  white: number;
  medium: number;
  dark: number;
  decaf: number;
  total: number;
}
