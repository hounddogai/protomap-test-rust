use super::*;

use acme_kit::text;

/// How screens and exports show a pending slo forecast.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloForecastView {
    pub code_text: String,
    pub title_text: String,
    pub category: String,
    pub checksum: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_pending_slo_forecast_view(item: &PendingSloForecast) -> PendingSloForecastView {
    PendingSloForecastView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        category: item.category.clone(),
        checksum: text::truncate(&item.checksum, 40),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_slo_forecast_views(items: &[PendingSloForecast]) -> Vec<PendingSloForecastView> {
    let mut views: Vec<PendingSloForecastView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_slo_forecast_view(item));
    }
    views
}

pub fn sum_pending_slo_forecast_view(view: &PendingSloForecastView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
