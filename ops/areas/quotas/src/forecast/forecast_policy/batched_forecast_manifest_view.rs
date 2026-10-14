use super::*;

use acme_kit::text;

/// How screens and exports show a batched forecast manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedForecastManifestView {
    pub code: String,
    pub title: String,
    pub discount_text: String,
    pub country: String,
    pub external_ref: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_batched_forecast_manifest_view(item: &BatchedForecastManifest) -> BatchedForecastManifestView {
    BatchedForecastManifestView {
        code: item.code.clone(),
        title: item.title.clone(),
        discount_text: format!("{:.2}", item.discount),
        country: text::truncate(&item.country, 80),
        external_ref: text::truncate(&item.external_ref, 40),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_forecast_manifest_views(items: &[BatchedForecastManifest]) -> Vec<BatchedForecastManifestView> {
    let mut views: Vec<BatchedForecastManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_forecast_manifest_view(item));
    }
    views
}

pub fn sum_batched_forecast_manifest_view(view: &BatchedForecastManifestView) -> Result<f64, String> {
    let parsed_discount = view
        .discount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_discount + parsed_maximum)
}
