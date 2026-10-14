use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal measurement manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMeasurementManifestView {
    pub code_text: String,
    pub title: String,
    pub notes: String,
    pub visible: String,
    pub label_text: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_seasonal_measurement_manifest_view(item: &SeasonalMeasurementManifest) -> SeasonalMeasurementManifestView {
    SeasonalMeasurementManifestView {
        code_text: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        notes: item.notes.clone(),
        visible: format!("{}", item.visible),
        label_text: text::truncate(&item.label, 40),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_measurement_manifest_views(
    items: &[SeasonalMeasurementManifest],
) -> Vec<SeasonalMeasurementManifestView> {
    let mut views: Vec<SeasonalMeasurementManifestView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_measurement_manifest_view(item));
    }
    views
}

pub fn sum_seasonal_measurement_manifest_view(view: &SeasonalMeasurementManifestView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
