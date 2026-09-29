use crate::models::{ExportSettings, RawAmounts, RoastTotals};

fn round_up(value: f64, increment: f64) -> f64 {
    if increment <= 0.0 {
        value
    } else {
        (value / increment).ceil() * increment
    }
}

pub fn validate_settings(settings: &ExportSettings) -> Result<(), String> {
    let factors = [
        ("White", settings.factors.white),
        ("Medium", settings.factors.medium),
        ("Dark", settings.factors.dark),
        ("Decaf", settings.factors.decaf),
    ];
    for (name, value) in factors {
        if !value.is_finite() || value <= 0.0 {
            return Err(format!("{name} ratio must be greater than zero."));
        }
    }

    if !settings.rounding_increment.is_finite() || settings.rounding_increment < 0.0 {
        return Err("Rounding increment cannot be negative.".into());
    }
    if settings.summary_position != "header" && settings.summary_position != "footer" {
        return Err("Summary position must be header or footer.".into());
    }
    Ok(())
}

pub fn calculate_raw(
    totals: &RoastTotals,
    settings: &ExportSettings,
) -> Result<RawAmounts, String> {
    validate_settings(settings)?;
    let increment = settings.rounding_increment;

    let white = round_up(totals.white as f64 / settings.factors.white, increment);
    let medium = round_up(totals.medium as f64 / settings.factors.medium, increment);
    let dark = round_up(
        (totals.dark + totals.espresso) as f64 / settings.factors.dark,
        increment,
    );
    let decaf = round_up(totals.decaf as f64 / settings.factors.decaf, increment);

    Ok(RawAmounts {
        white,
        medium,
        dark,
        decaf,
        total: white + medium + dark + decaf,
    })
}

#[tauri::command]
pub fn calculate_requirements(
    totals: RoastTotals,
    settings: ExportSettings,
) -> Result<RawAmounts, String> {
    calculate_raw(&totals, &settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Factors;

    fn settings(rounding_increment: f64) -> ExportSettings {
        ExportSettings {
            factors: Factors::default(),
            rounding_increment,
            summary_position: "footer".into(),
        }
    }

    #[test]
    fn combines_dark_and_espresso_for_raw_dark() {
        let totals = RoastTotals {
            dark: 10,
            espresso: 2,
            ..Default::default()
        };
        let raw = calculate_raw(&totals, &settings(0.0)).unwrap();
        assert!((raw.dark - (12.0 / 1.10)).abs() < 0.0001);
    }

    #[test]
    fn rounds_each_roast_up_independently() {
        let totals = RoastTotals {
            white: 1,
            medium: 1,
            dark: 1,
            decaf: 1,
            ..Default::default()
        };
        let raw = calculate_raw(&totals, &settings(0.5)).unwrap();
        assert_eq!(raw.white, 1.0);
        assert_eq!(raw.medium, 1.0);
        assert_eq!(raw.dark, 1.0);
        assert_eq!(raw.decaf, 1.0);
        assert_eq!(raw.total, 4.0);
    }
}
