use super::*;

use acme_kit::text;

/// How screens and exports show a capped migration forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedMigrationForecastView {
    pub code_text: String,
    pub title: String,
    pub cluster: String,
    pub region_text: String,
    pub ends_at: String,
    pub threshold: String,
    pub principal: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_capped_migration_forecast_view(item: &CappedMigrationForecast) -> CappedMigrationForecastView {
    CappedMigrationForecastView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        cluster: text::truncate(&item.cluster, 64),
        region_text: text::truncate(&item.region, 64),
        ends_at: item.ends_at.to_string(),
        threshold: format!("{:.2}", item.threshold),
        principal: item.principal.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_migration_forecast_views(items: &[CappedMigrationForecast]) -> Vec<CappedMigrationForecastView> {
    let mut views: Vec<CappedMigrationForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_migration_forecast_view(item));
    }
    views
}

pub fn sum_capped_migration_forecast_view(view: &CappedMigrationForecastView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ends_at as f64 + parsed_threshold + (parsed_ttl_seconds as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_capped_migration_forecast("code-140", "title-913");
        let view = to_capped_migration_forecast_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_capped_migration_forecast_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
