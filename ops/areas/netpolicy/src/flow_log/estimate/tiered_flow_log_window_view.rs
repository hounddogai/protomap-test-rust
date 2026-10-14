use super::*;

use acme_kit::text;

/// How screens and exports show a tiered flow log window.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFlowLogWindowView {
    pub code: String,
    pub title_text: String,
    pub retries: String,
    pub capacity: String,
    pub category: String,
    pub weight: String,
    pub country: String,
    pub floor_value: String,
    pub ceiling_text: String,
    pub unit: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_tiered_flow_log_window_view(item: &TieredFlowLogWindow) -> TieredFlowLogWindowView {
    TieredFlowLogWindowView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        retries: item.retries.to_string(),
        capacity: item.capacity.to_string(),
        category: text::truncate(&item.category, 40),
        weight: format!("{:.2}", item.weight),
        country: text::truncate(&item.country, 80),
        floor_value: format!("{:.2}", item.floor_value),
        ceiling_text: format!("{:.2}", item.ceiling),
        unit: item.unit.clone(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_flow_log_window_views(items: &[TieredFlowLogWindow]) -> Vec<TieredFlowLogWindowView> {
    let mut views: Vec<TieredFlowLogWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_flow_log_window_view(item));
    }
    views
}

pub fn sum_tiered_flow_log_window_view(view: &TieredFlowLogWindowView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_capacity as f64) + parsed_weight)
}
