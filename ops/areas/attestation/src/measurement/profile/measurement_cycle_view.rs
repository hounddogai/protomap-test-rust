use super::*;

use acme_kit::text;

/// How screens and exports show a measurement cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementCycleView {
    pub code_text: String,
    pub name: String,
    pub label: String,
    pub category_text: String,
    pub issuer: String,
    pub limit: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_measurement_cycle_view(item: &MeasurementCycle) -> MeasurementCycleView {
    MeasurementCycleView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        label: item.label.clone(),
        category_text: item.category.clone(),
        issuer: text::truncate(&item.issuer, 64),
        limit: item.limit.to_string(),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_cycle_views(items: &[MeasurementCycle]) -> Vec<MeasurementCycleView> {
    let mut views: Vec<MeasurementCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_cycle_view(item));
    }
    views
}

pub fn sum_measurement_cycle_view(view: &MeasurementCycleView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_limit as f64 + parsed_ratio)
}
