use super::*;

use acme_kit::text;

/// How screens and exports show a egress metric.
#[derive(Debug, Clone, PartialEq)]
pub struct EgressMetricView {
    pub code: String,
    pub label: String,
    pub pinned: String,
    pub title: String,
    pub cost: String,
    pub cipher: String,
    pub locale_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_egress_metric_view(item: &EgressMetric) -> EgressMetricView {
    EgressMetricView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        pinned: format!("{}", item.pinned),
        title: item.title.clone(),
        cost: format!("{:.2}", item.cost),
        cipher: item.cipher.clone(),
        locale_text: item.locale.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_egress_metric_views(items: &[EgressMetric]) -> Vec<EgressMetricView> {
    let mut views: Vec<EgressMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_egress_metric_view(item));
    }
    views
}

pub fn sum_egress_metric_view(view: &EgressMetricView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_cost + (parsed_quantity as f64))
}
