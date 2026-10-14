use super::*;

/// How screens and exports show a tiered probe checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeCheckpointView {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub quota_units_text: String,
    pub minimum_text: String,
    pub approved: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_tiered_probe_checkpoint_view(item: &TieredProbeCheckpoint) -> TieredProbeCheckpointView {
    TieredProbeCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        currency: item.currency.clone(),
        quota_units_text: item.quota_units.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        approved: format!("{}", item.approved),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_probe_checkpoint_views(items: &[TieredProbeCheckpoint]) -> Vec<TieredProbeCheckpointView> {
    let mut views: Vec<TieredProbeCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_probe_checkpoint_view(item));
    }
    views
}

pub fn sum_tiered_probe_checkpoint_view(view: &TieredProbeCheckpointView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_minimum + (parsed_limit as f64))
}
