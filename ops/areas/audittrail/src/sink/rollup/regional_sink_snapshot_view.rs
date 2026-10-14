use super::*;

use acme_kit::text;

/// How screens and exports show a regional sink snapshot.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSinkSnapshotView {
    pub code: String,
    pub label: String,
    pub updated_at: String,
    pub minimum: String,
    pub summary: String,
    pub burst: String,
    pub port: String,
    pub locale: String,
    pub retries: String,
    pub budget_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_regional_sink_snapshot_view(item: &RegionalSinkSnapshot) -> RegionalSinkSnapshotView {
    RegionalSinkSnapshotView {
        code: text::truncate(&item.code, 80),
        label: item.label.clone(),
        updated_at: item.updated_at.to_string(),
        minimum: format!("{:.2}", item.minimum),
        summary: item.summary.clone(),
        burst: item.burst.to_string(),
        port: item.port.to_string(),
        locale: item.locale.clone(),
        retries: item.retries.to_string(),
        budget_text: format!("{:.2}", item.budget),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_sink_snapshot_views(items: &[RegionalSinkSnapshot]) -> Vec<RegionalSinkSnapshotView> {
    let mut views: Vec<RegionalSinkSnapshotView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_sink_snapshot_view(item));
    }
    views
}

pub fn sum_regional_sink_snapshot_view(view: &RegionalSinkSnapshotView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_minimum + (parsed_burst as f64))
}
