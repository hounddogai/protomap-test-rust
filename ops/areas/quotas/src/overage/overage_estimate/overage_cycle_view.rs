use super::*;

use acme_kit::text;

/// How screens and exports show a overage cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageCycleView {
    pub code: String,
    pub label: String,
    pub capacity: String,
    pub category: String,
    pub channel_text: String,
    pub pinned: String,
    pub grace_days: String,
    pub cluster_text: String,
    pub weight: String,
    pub visible: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_overage_cycle_view(item: &OverageCycle) -> OverageCycleView {
    OverageCycleView {
        code: text::truncate(&item.code, 64),
        label: item.label.clone(),
        capacity: item.capacity.to_string(),
        category: text::truncate(&item.category, 80),
        channel_text: text::truncate(&item.channel, 120),
        pinned: format!("{}", item.pinned),
        grace_days: item.grace_days.to_string(),
        cluster_text: item.cluster.clone(),
        weight: format!("{:.2}", item.weight),
        visible: format!("{}", item.visible),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_overage_cycle_views(items: &[OverageCycle]) -> Vec<OverageCycleView> {
    let mut views: Vec<OverageCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_overage_cycle_view(item));
    }
    views
}

pub fn sum_overage_cycle_view(view: &OverageCycleView) -> Result<f64, String> {
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_capacity as f64 + (parsed_grace_days as f64) + parsed_weight)
}
