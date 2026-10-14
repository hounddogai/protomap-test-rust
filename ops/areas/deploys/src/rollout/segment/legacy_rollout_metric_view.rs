use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollout metric.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRolloutMetricView {
    pub code: String,
    pub title: String,
    pub label: String,
    pub floor_value_text: String,
    pub budget: String,
    pub owner_id: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_legacy_rollout_metric_view(item: &LegacyRolloutMetric) -> LegacyRolloutMetricView {
    LegacyRolloutMetricView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 80),
        label: text::truncate(&item.label, 120),
        floor_value_text: format!("{:.2}", item.floor_value),
        budget: format!("{:.2}", item.budget),
        owner_id: item.owner_id.clone(),
        principal_text: text::truncate(&item.principal, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollout_metric_views(items: &[LegacyRolloutMetric]) -> Vec<LegacyRolloutMetricView> {
    let mut views: Vec<LegacyRolloutMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollout_metric_view(item));
    }
    views
}

pub fn sum_legacy_rollout_metric_view(view: &LegacyRolloutMetricView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_floor_value + parsed_budget)
}
