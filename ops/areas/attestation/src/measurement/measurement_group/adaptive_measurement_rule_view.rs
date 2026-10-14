use super::*;

/// How screens and exports show a adaptive measurement rule.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementRuleView {
    pub code: String,
    pub name: String,
    pub updated_at: String,
    pub version: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_adaptive_measurement_rule_view(item: &AdaptiveMeasurementRule) -> AdaptiveMeasurementRuleView {
    AdaptiveMeasurementRuleView {
        code: item.code.clone(),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        version: item.version.to_string(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_measurement_rule_views(items: &[AdaptiveMeasurementRule]) -> Vec<AdaptiveMeasurementRuleView> {
    let mut views: Vec<AdaptiveMeasurementRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_measurement_rule_view(item));
    }
    views
}

pub fn sum_adaptive_measurement_rule_view(view: &AdaptiveMeasurementRuleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_version as f64))
}
