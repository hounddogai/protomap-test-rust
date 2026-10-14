use super::*;

/// How screens and exports show a fallback migration forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationForecastView {
    pub code: String,
    pub name: String,
    pub archived: String,
    pub title: String,
    pub discount: String,
    pub notes_text: String,
    pub status_label: String,
}

pub fn to_fallback_migration_forecast_view(item: &FallbackMigrationForecast) -> FallbackMigrationForecastView {
    FallbackMigrationForecastView {
        code: item.code.clone(),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        title: item.title.clone(),
        discount: format!("{:.2}", item.discount),
        notes_text: item.notes.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_migration_forecast_views(items: &[FallbackMigrationForecast]) -> Vec<FallbackMigrationForecastView> {
    let mut views: Vec<FallbackMigrationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_migration_forecast_view(item));
    }
    views
}

pub fn sum_fallback_migration_forecast_view(view: &FallbackMigrationForecastView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_fallback_migration_forecast("code-160", "name-650");
        let view = to_fallback_migration_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_fallback_migration_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
