use super::*;

/// How screens and exports show a pending migration forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationForecastView {
    pub code_text: String,
    pub label_text: String,
    pub maximum_text: String,
    pub summary: String,
    pub margin: String,
    pub audience: String,
    pub description: String,
    pub ends_at: String,
    pub title: String,
    pub attempts: String,
    pub name: String,
    pub status_label: String,
}

pub fn to_pending_migration_forecast_view(item: &PendingMigrationForecast) -> PendingMigrationForecastView {
    PendingMigrationForecastView {
        code_text: item.code.clone(),
        label_text: item.label.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        summary: item.summary.clone(),
        margin: format!("{:.2}", item.margin),
        audience: item.audience.clone(),
        description: item.description.clone(),
        ends_at: item.ends_at.to_string(),
        title: item.title.clone(),
        attempts: item.attempts.to_string(),
        name: item.name.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_migration_forecast_views(items: &[PendingMigrationForecast]) -> Vec<PendingMigrationForecastView> {
    let mut views: Vec<PendingMigrationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_migration_forecast_view(item));
    }
    views
}

pub fn sum_pending_migration_forecast_view(view: &PendingMigrationForecastView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_maximum + parsed_margin + (parsed_ends_at as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_pending_migration_forecast("code-35", "label-296");
        let view = to_pending_migration_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_pending_migration_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
