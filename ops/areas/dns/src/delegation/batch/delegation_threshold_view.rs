use super::*;

use acme_kit::text;

/// How screens and exports show a delegation threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationThresholdView {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub category: String,
    pub floor_value: String,
    pub description_text: String,
    pub limit: String,
    pub reviewer: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_delegation_threshold_view(item: &DelegationThreshold) -> DelegationThresholdView {
    DelegationThresholdView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        cluster: item.cluster.clone(),
        category: item.category.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        description_text: text::truncate(&item.description, 64),
        limit: item.limit.to_string(),
        reviewer: text::truncate(&item.reviewer, 80),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_delegation_threshold_views(items: &[DelegationThreshold]) -> Vec<DelegationThresholdView> {
    let mut views: Vec<DelegationThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_delegation_threshold_view(item));
    }
    views
}

pub fn sum_delegation_threshold_view(view: &DelegationThresholdView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_floor_value + (parsed_limit as f64))
}
