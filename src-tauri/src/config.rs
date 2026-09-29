use crate::models::{AppConfig, ConfigResponse};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

const DEFAULT_CONFIG: &str = include_str!("../../config/labels.json");
const VALID_CATEGORIES: [&str; 5] = ["white", "medium", "dark", "espresso", "decaf"];

fn parse_and_validate(contents: &str) -> Result<AppConfig, String> {
    let config: AppConfig =
        serde_json::from_str(contents).map_err(|error| format!("Invalid JSON: {error}"))?;

    if config.product_mappings.is_empty() {
        return Err("productMappings must contain at least one product name".into());
    }

    for (product, category) in &config.product_mappings {
        if product.trim().is_empty() {
            return Err("Product names in productMappings cannot be empty".into());
        }
        if !VALID_CATEGORIES.contains(&category.as_str()) {
            return Err(format!(
                "Product \"{product}\" uses invalid category \"{category}\""
            ));
        }
    }

    Ok(config)
}

fn external_config_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("labels.json"))
        .map_err(|error| format!("Could not locate the application config directory: {error}"))
}

#[tauri::command]
pub fn load_config(app: AppHandle) -> Result<ConfigResponse, String> {
    let default = parse_and_validate(DEFAULT_CONFIG)?;
    let path = external_config_path(&app)?;

    if !path.exists() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("Could not create config directory: {error}"))?;
        }
        fs::write(&path, DEFAULT_CONFIG)
            .map_err(|error| format!("Could not create editable label config: {error}"))?;
    }

    match fs::read_to_string(&path) {
        Ok(contents) => match parse_and_validate(&contents) {
            Ok(config) => {
                let missing_total_bags = serde_json::from_str::<serde_json::Value>(&contents)
                    .ok()
                    .and_then(|value| value.pointer("/labels/totalBags").cloned())
                    .is_none();
                let warning = if missing_total_bags {
                    serde_json::to_string_pretty(&config)
                        .map_err(|error| error.to_string())
                        .and_then(|updated| fs::write(&path, updated).map_err(|error| error.to_string()))
                        .err()
                        .map(|error| format!("Could not add the new total-bags label to the editable config: {error}"))
                } else {
                    None
                };
                Ok(ConfigResponse {
                    config,
                    config_path: path.to_string_lossy().into_owned(),
                    warning,
                })
            }
            Err(error) => Ok(ConfigResponse {
                config: default,
                config_path: path.to_string_lossy().into_owned(),
                warning: Some(format!(
                    "The external labels file is invalid ({error}). Bundled defaults are being used."
                )),
            }),
        },
        Err(error) => Ok(ConfigResponse {
            config: default,
            config_path: path.to_string_lossy().into_owned(),
            warning: Some(format!(
                "The external labels file could not be read ({error}). Bundled defaults are being used."
            )),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_config_is_valid() {
        let config = parse_and_validate(DEFAULT_CONFIG).expect("default config should parse");
        assert!(config
            .product_mappings
            .keys()
            .any(|name| name.contains("Espresso")));
    }

    #[test]
    fn rejects_unknown_category() {
        let invalid = DEFAULT_CONFIG.replace("\": \"white\"", "\": \"blonde\"");
        assert!(parse_and_validate(&invalid).is_err());
    }
}
