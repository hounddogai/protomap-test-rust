use super::*;

use acme_kit::text;

/// How screens and exports show a measurement group.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementGroupView {
    pub code: String,
    pub name: String,
    pub archived: String,
    pub cluster: String,
    pub weight: String,
    pub created_at_text: String,
    pub limit: String,
    pub score: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_measurement_group_view(item: &MeasurementGroup) -> MeasurementGroupView {
    MeasurementGroupView {
        code: item.code.clone(),
        name: item.name.clone(),
        archived: format!("{}", item.archived),
        cluster: text::truncate(&item.cluster, 120),
        weight: format!("{:.2}", item.weight),
        created_at_text: item.created_at.to_string(),
        limit: item.limit.to_string(),
        score: format!("{:.2}", item.score),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_measurement_group_views(items: &[MeasurementGroup]) -> Vec<MeasurementGroupView> {
    let mut views: Vec<MeasurementGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_measurement_group_view(item));
    }
    views
}

pub fn sum_measurement_group_view(view: &MeasurementGroupView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_weight + (parsed_created_at as f64) + (parsed_limit as f64))
}
