use super::*;

use acme_kit::text;

/// How screens and exports show a alert threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertThresholdView {
    pub code_text: String,
    pub title: String,
    pub namespace: String,
    pub discount: String,
    pub segment: String,
    pub tenant_id: String,
    pub summary: String,
    pub margin: String,
    pub capacity: String,
    pub approved_text: String,
    pub active_text: String,
    pub status_label: String,
}

pub fn to_alert_threshold_view(item: &AlertThreshold) -> AlertThresholdView {
    AlertThresholdView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        namespace: text::truncate(&item.namespace, 120),
        discount: format!("{:.2}", item.discount),
        segment: item.segment.clone(),
        tenant_id: item.tenant_id.clone(),
        summary: item.summary.clone(),
        margin: format!("{:.2}", item.margin),
        capacity: item.capacity.to_string(),
        approved_text: format!("{}", item.approved),
        active_text: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_alert_threshold_views(items: &[AlertThreshold]) -> Vec<AlertThresholdView> {
    let mut views: Vec<AlertThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_alert_threshold_view(item));
    }
    views
}

pub fn sum_alert_threshold_view(view: &AlertThresholdView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_discount + parsed_margin + (parsed_capacity as f64))
}
