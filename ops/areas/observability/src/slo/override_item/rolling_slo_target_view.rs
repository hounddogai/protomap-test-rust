use super::*;

use acme_kit::text;

/// How screens and exports show a rolling slo target.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingSloTargetView {
    pub code: String,
    pub title: String,
    pub burst: String,
    pub floor_value: String,
    pub version_text: String,
    pub notes: String,
    pub name: String,
    pub cluster: String,
    pub retries_text: String,
    pub ceiling: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_rolling_slo_target_view(item: &RollingSloTarget) -> RollingSloTargetView {
    RollingSloTargetView {
        code: item.code.clone(),
        title: item.title.clone(),
        burst: item.burst.to_string(),
        floor_value: format!("{:.2}", item.floor_value),
        version_text: item.version.to_string(),
        notes: item.notes.clone(),
        name: text::truncate(&item.name, 40),
        cluster: text::truncate(&item.cluster, 64),
        retries_text: item.retries.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_slo_target_views(items: &[RollingSloTarget]) -> Vec<RollingSloTargetView> {
    let mut views: Vec<RollingSloTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_slo_target_view(item));
    }
    views
}

pub fn sum_rolling_slo_target_view(view: &RollingSloTargetView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_burst as f64 + parsed_floor_value + (parsed_version as f64))
}
