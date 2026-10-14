use super::*;

use acme_kit::text;

/// How screens and exports show a timeline group.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroupView {
    pub code: String,
    pub label: String,
    pub cost: String,
    pub issuer_text: String,
    pub expires_at: String,
    pub quantity_text: String,
    pub ttl_seconds_text: String,
    pub margin: String,
    pub status_label: String,
}

pub fn to_timeline_group_view(item: &TimelineGroup) -> TimelineGroupView {
    TimelineGroupView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 120),
        cost: format!("{:.2}", item.cost),
        issuer_text: item.issuer.clone(),
        expires_at: item.expires_at.to_string(),
        quantity_text: item.quantity.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        margin: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_timeline_group_views(items: &[TimelineGroup]) -> Vec<TimelineGroupView> {
    let mut views: Vec<TimelineGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_timeline_group_view(item));
    }
    views
}

pub fn sum_timeline_group_view(view: &TimelineGroupView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_cost + (parsed_expires_at as f64) + (parsed_quantity as f64))
}
