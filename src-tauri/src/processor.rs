use crate::models::{AppConfig, CustomerSummary, ProcessedData, RoastTotals};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Deserialize)]
struct CsvRow {
    #[serde(rename = "Name")]
    order_name: String,
    #[serde(rename = "Shipping Name")]
    shipping_name: String,
    #[serde(rename = "Lineitem quantity")]
    quantity: String,
    #[serde(rename = "Lineitem name")]
    product_name: String,
}

pub fn process_reader<R: std::io::Read>(
    reader: R,
    source_path: String,
    config: &AppConfig,
) -> Result<ProcessedData, String> {
    let mut csv = csv::ReaderBuilder::new().flexible(true).from_reader(reader);
    let rows: Vec<CsvRow> = csv
        .deserialize()
        .enumerate()
        .map(|(index, result)| {
            result.map_err(|error| format!("Could not read CSV row {}: {error}", index + 2))
        })
        .collect::<Result<_, _>>()?;

    let mut names_by_order = BTreeMap::new();
    for row in &rows {
        let order = row.order_name.trim();
        let shipping_name = row.shipping_name.trim();
        if !order.is_empty() && !shipping_name.is_empty() {
            names_by_order.insert(order.to_owned(), shipping_name.to_owned());
        }
    }

    let mut customers: BTreeMap<String, RoastTotals> = BTreeMap::new();
    let mut unmatched = BTreeSet::new();
    let mut warnings = Vec::new();
    let mut line_item_count = 0;

    for (index, row) in rows.iter().enumerate() {
        let product_name = row.product_name.trim();
        if product_name.is_empty() {
            continue;
        }

        let quantity = row.quantity.trim().parse::<u32>().map_err(|_| {
            format!(
                "Lineitem quantity on CSV row {} is not a whole number: \"{}\"",
                index + 2,
                row.quantity
            )
        })?;

        let shipping_name = if !row.shipping_name.trim().is_empty() {
            row.shipping_name.trim()
        } else {
            names_by_order
                .get(row.order_name.trim())
                .map(String::as_str)
                .unwrap_or("")
        };

        let shipping_name = if shipping_name.is_empty() {
            warnings.push(format!(
                "CSV row {} has no Shipping Name; it was grouped under Unknown Customer.",
                index + 2
            ));
            "Unknown Customer"
        } else {
            shipping_name
        };

        let category = config
            .product_mappings
            .get(product_name)
            .map(String::as_str)
            .unwrap_or("other");

        if category == "other" {
            unmatched.insert(product_name.to_owned());
        }

        customers
            .entry(shipping_name.to_owned())
            .or_default()
            .add(category, quantity);
        line_item_count += 1;
    }

    if line_item_count == 0 {
        return Err("The CSV contains no line items to process.".into());
    }

    let customers: Vec<CustomerSummary> = customers
        .into_iter()
        .map(|(shipping_name, totals)| CustomerSummary {
            shipping_name,
            totals,
        })
        .collect();

    let mut totals = RoastTotals::default();
    for customer in &customers {
        totals.merge(&customer.totals);
    }

    Ok(ProcessedData {
        source_path,
        customers,
        totals,
        unmatched_products: unmatched.into_iter().collect(),
        warnings,
        line_item_count,
    })
}

#[tauri::command]
pub fn process_csv(path: String, config: AppConfig) -> Result<ProcessedData, String> {
    let file = std::fs::File::open(&path)
        .map_err(|error| format!("Could not open the selected CSV: {error}"))?;
    process_reader(file, path, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Labels;

    fn config() -> AppConfig {
        let mappings = [
            ("Dark Roast - 12 oz", "dark"),
            ("Espresso Roast - 12 oz", "espresso"),
            ("White Roast - 12 oz", "white"),
        ]
        .into_iter()
        .map(|(name, category)| (name.to_owned(), category.to_owned()))
        .collect();

        AppConfig {
            labels: Labels {
                app_title: "Test".into(),
                import_button: "Import".into(),
                export_button: "Export".into(),
                shipping_name: "Shipping Name".into(),
                white: "White".into(),
                medium: "Medium".into(),
                dark: "Dark".into(),
                espresso: "Espresso".into(),
                decaf: "Decaf".into(),
                other: "Other".into(),
                bag_totals: "Bags".into(),
                total_bags: "Total Bags".into(),
                raw_amounts: "Raw".into(),
                raw_white: "Raw White".into(),
                raw_medium: "Raw Medium".into(),
                raw_dark: "Raw Dark".into(),
                raw_decaf: "Raw Decaf".into(),
                raw_total: "Total Raw".into(),
            },
            product_mappings: mappings,
        }
    }

    #[test]
    fn associates_continuation_rows_and_combines_customer_orders() {
        let input = concat!(
            "Name,Shipping Name,Lineitem quantity,Lineitem name\n",
            "#1,Alex Example,2,Dark Roast - 12 oz\n",
            "#1,,1,Espresso Roast - 12 oz\n",
            "#2,Alex Example,3,White Roast - 12 oz\n",
        );

        let result = process_reader(input.as_bytes(), "test.csv".into(), &config()).unwrap();
        assert_eq!(result.customers.len(), 1);
        assert_eq!(result.customers[0].totals.dark, 2);
        assert_eq!(result.customers[0].totals.espresso, 1);
        assert_eq!(result.customers[0].totals.white, 3);
    }

    #[test]
    fn puts_unknown_products_in_other() {
        let input = concat!(
            "Name,Shipping Name,Lineitem quantity,Lineitem name\n",
            "#1,Alex Example,4,Seasonal Roast\n",
        );

        let result = process_reader(input.as_bytes(), "test.csv".into(), &config()).unwrap();
        assert_eq!(result.totals.other, 4);
        assert_eq!(result.unmatched_products, vec!["Seasonal Roast"]);
    }

    #[test]
    fn processes_synthetic_shopify_export() {
        let input = include_str!("../tests/fixtures/sample_orders.csv");
        let full_config: AppConfig =
            serde_json::from_str(include_str!("../../config/labels.json")).unwrap();
        let result = process_reader(
            input.as_bytes(),
            "sample_orders.csv".into(),
            &full_config,
        )
        .unwrap();

        assert_eq!(result.totals.white, 6);
        assert_eq!(result.totals.medium, 19);
        assert_eq!(result.totals.dark, 49);
        assert_eq!(result.totals.espresso, 5);
        assert_eq!(result.totals.decaf, 6);
        assert_eq!(result.totals.other, 0);
        assert_eq!(result.totals.total_bags(), 85);
    }
}
