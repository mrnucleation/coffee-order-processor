use crate::{
    calculation::calculate_raw,
    models::{AppConfig, ExportSettings, ProcessedData, RawAmounts, RoastTotals},
};
use rust_xlsxwriter::{Color, Format, FormatAlign, Workbook, Worksheet};

fn write_summary(
    worksheet: &mut Worksheet,
    start_row: u32,
    totals: &RoastTotals,
    raw: &RawAmounts,
    config: &AppConfig,
    title_format: &Format,
    header_format: &Format,
    bag_total_format: &Format,
    raw_total_format: &Format,
) -> Result<(), rust_xlsxwriter::XlsxError> {
    let labels = &config.labels;
    worksheet.merge_range(start_row, 0, start_row, 7, &labels.bag_totals, title_format)?;

    let bag_labels = [
        &labels.white,
        &labels.medium,
        &labels.dark,
        &labels.espresso,
        &labels.decaf,
        &labels.other,
        &labels.total_bags,
    ];
    let bag_values = [
        totals.white,
        totals.medium,
        totals.dark,
        totals.espresso,
        totals.decaf,
        totals.other,
        totals.total_bags(),
    ];
    for (index, label) in bag_labels.iter().enumerate() {
        let column = index as u16 + 1;
        worksheet.write_string_with_format(start_row + 1, column, *label, header_format)?;
        if bag_values[index] == 0 {
            worksheet.write_blank(start_row + 2, column, bag_total_format)?;
        } else {
            worksheet.write_number_with_format(
                start_row + 2,
                column,
                bag_values[index] as f64,
                bag_total_format,
            )?;
        }
    }

    worksheet.merge_range(
        start_row + 4,
        0,
        start_row + 4,
        7,
        &labels.raw_amounts,
        title_format,
    )?;
    let raw_labels = [
        &labels.raw_white,
        &labels.raw_medium,
        &labels.raw_dark,
        &labels.raw_decaf,
        &labels.raw_total,
    ];
    let raw_values = [raw.white, raw.medium, raw.dark, raw.decaf, raw.total];
    for (index, label) in raw_labels.iter().enumerate() {
        let column = index as u16 + 1;
        worksheet.write_string_with_format(start_row + 5, column, *label, header_format)?;
        if raw_values[index] == 0.0 {
            worksheet.write_blank(start_row + 6, column, raw_total_format)?;
        } else {
            worksheet.write_number_with_format(
                start_row + 6,
                column,
                raw_values[index],
                raw_total_format,
            )?;
        }
    }
    Ok(())
}

pub fn export_workbook(
    path: &str,
    data: &ProcessedData,
    settings: &ExportSettings,
    config: &AppConfig,
) -> Result<(), String> {
    let raw = calculate_raw(&data.totals, settings)?;
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();
    worksheet
        .set_name("Coffee Orders")
        .map_err(|error| error.to_string())?;

    let title_format = Format::new()
        .set_bold()
        .set_font_size(14)
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x5A3825))
        .set_align(FormatAlign::Center);
    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x7B5138))
        .set_align(FormatAlign::Center);
    let raw_total_format = Format::new()
        .set_bold()
        .set_background_color(Color::RGB(0xE9DDC8))
        .set_align(FormatAlign::Center)
        .set_num_format("0.00");
    let integer_format = Format::new()
        .set_align(FormatAlign::Center)
        .set_num_format("0");
    let bag_total_format = integer_format
        .clone()
        .set_bold()
        .set_background_color(Color::RGB(0xE9DDC8));

    let table_header_row = if settings.summary_position == "header" {
        write_summary(
            worksheet,
            0,
            &data.totals,
            &raw,
            config,
            &title_format,
            &header_format,
            &bag_total_format,
            &raw_total_format,
        )
        .map_err(|error| error.to_string())?;
        8
    } else {
        0
    };

    let headers = [
        &config.labels.shipping_name,
        &config.labels.white,
        &config.labels.medium,
        &config.labels.dark,
        &config.labels.espresso,
        &config.labels.decaf,
        &config.labels.other,
    ];
    for (column, header) in headers.iter().enumerate() {
        worksheet
            .write_string_with_format(table_header_row, column as u16, *header, &header_format)
            .map_err(|error| error.to_string())?;
    }

    for (index, customer) in data.customers.iter().enumerate() {
        let row = table_header_row + index as u32 + 1;
        worksheet
            .write_string(row, 0, &customer.shipping_name)
            .map_err(|error| error.to_string())?;
        let values = [
            customer.totals.white,
            customer.totals.medium,
            customer.totals.dark,
            customer.totals.espresso,
            customer.totals.decaf,
            customer.totals.other,
        ];
        for (offset, value) in values.iter().enumerate() {
            if *value == 0 {
                worksheet
                    .write_blank(row, offset as u16 + 1, &integer_format)
                    .map_err(|error| error.to_string())?;
            } else {
                worksheet
                    .write_number_with_format(
                        row,
                        offset as u16 + 1,
                        *value as f64,
                        &integer_format,
                    )
                    .map_err(|error| error.to_string())?;
            }
        }
    }

    let table_last_row = table_header_row + data.customers.len() as u32;
    worksheet
        .autofilter(table_header_row, 0, table_last_row, 6)
        .map_err(|error| error.to_string())?;
    worksheet
        .set_freeze_panes(table_header_row + 1, 1)
        .map_err(|error| error.to_string())?;
    worksheet
        .set_column_width(0, 28)
        .map_err(|error| error.to_string())?;
    for column in 1..=6 {
        worksheet
            .set_column_width(column, 14)
            .map_err(|error| error.to_string())?;
    }
    worksheet
        .set_column_width(7, 22)
        .map_err(|error| error.to_string())?;

    if settings.summary_position == "footer" {
        write_summary(
            worksheet,
            table_last_row + 2,
            &data.totals,
            &raw,
            config,
            &title_format,
            &header_format,
            &bag_total_format,
            &raw_total_format,
        )
        .map_err(|error| error.to_string())?;
    }

    workbook
        .save(path)
        .map_err(|error| format!("Could not save the Excel workbook: {error}"))
}

#[tauri::command]
pub fn export_excel(
    path: String,
    data: ProcessedData,
    settings: ExportSettings,
    config: AppConfig,
) -> Result<(), String> {
    export_workbook(&path, &data, &settings, &config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::Factors, processor::process_reader};

    #[test]
    fn exports_sample_to_nonempty_workbook() {
        let config: AppConfig =
            serde_json::from_str(include_str!("../../config/labels.json")).unwrap();
        let data = process_reader(
            include_str!("../tests/fixtures/sample_orders.csv").as_bytes(),
            "sample_orders.csv".into(),
            &config,
        )
        .unwrap();
        let settings = ExportSettings {
            factors: Factors::default(),
            rounding_increment: 0.25,
            summary_position: "footer".into(),
        };
        let directory = tempfile::tempdir().unwrap();
        let output = directory.path().join("coffee-orders.xlsx");

        export_workbook(output.to_str().unwrap(), &data, &settings, &config).unwrap();

        assert!(output.exists());
        assert!(std::fs::metadata(output).unwrap().len() > 1_000);
    }
}
