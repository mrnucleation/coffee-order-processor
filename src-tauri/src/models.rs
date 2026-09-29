use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

fn default_total_bags_label() -> String {
    "Total Bags of Coffee".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub labels: Labels,
    pub product_mappings: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Labels {
    pub app_title: String,
    pub import_button: String,
    pub export_button: String,
    pub shipping_name: String,
    pub white: String,
    pub medium: String,
    pub dark: String,
    pub espresso: String,
    pub decaf: String,
    pub other: String,
    pub bag_totals: String,
    #[serde(default = "default_total_bags_label")]
    pub total_bags: String,
    pub raw_amounts: String,
    pub raw_white: String,
    pub raw_medium: String,
    pub raw_dark: String,
    pub raw_decaf: String,
    pub raw_total: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoastTotals {
    pub white: u32,
    pub medium: u32,
    pub dark: u32,
    pub espresso: u32,
    pub decaf: u32,
    pub other: u32,
}

impl RoastTotals {
    pub fn add(&mut self, category: &str, quantity: u32) {
        match category {
            "white" => self.white += quantity,
            "medium" => self.medium += quantity,
            "dark" => self.dark += quantity,
            "espresso" => self.espresso += quantity,
            "decaf" => self.decaf += quantity,
            _ => self.other += quantity,
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.white += other.white;
        self.medium += other.medium;
        self.dark += other.dark;
        self.espresso += other.espresso;
        self.decaf += other.decaf;
        self.other += other.other;
    }

    pub fn total_bags(&self) -> u32 {
        self.white + self.medium + self.dark + self.espresso + self.decaf + self.other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomerSummary {
    pub shipping_name: String,
    pub totals: RoastTotals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessedData {
    pub source_path: String,
    pub customers: Vec<CustomerSummary>,
    pub totals: RoastTotals,
    pub unmatched_products: Vec<String>,
    pub warnings: Vec<String>,
    pub line_item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Factors {
    pub white: f64,
    pub medium: f64,
    pub dark: f64,
    pub decaf: f64,
}

impl Default for Factors {
    fn default() -> Self {
        Self {
            white: 1.20,
            medium: 1.15,
            dark: 1.10,
            decaf: 1.125,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettings {
    pub factors: Factors,
    pub rounding_increment: f64,
    pub summary_position: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RawAmounts {
    pub white: f64,
    pub medium: f64,
    pub dark: f64,
    pub decaf: f64,
    pub total: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub config: AppConfig,
    pub config_path: String,
    pub warning: Option<String>,
}
