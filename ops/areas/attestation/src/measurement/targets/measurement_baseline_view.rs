use super::*;

use acme_kit::text;

/// How screens and exports show a measurement baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementBaselineView {
    pub code: String,
    pub name: String,
    pub ceiling_text: String,
    pub unit: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_measurement_baseline_view(item: &MeasurementBaseline) -> MeasurementBaselineView {
    MeasurementBaselineView {
        code: text::truncate(&item.code, 120),
        name: text::truncate(&item.name, 120),
        ceiling_text: format!("{:.2}", item.ceiling),
        unit: text::truncate(&item.unit, 80),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_baseline_views(items: &[MeasurementBaseline]) -> Vec<MeasurementBaselineView> {
    let mut views: Vec<MeasurementBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_baseline_view(item));
    }
    views
}

pub fn sum_measurement_baseline_view(view: &MeasurementBaselineView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_ceiling)
}
