use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive identity target.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentityTargetView {
    pub code_text: String,
    pub name: String,
    pub ends_at: String,
    pub capacity: String,
    pub segment: String,
    pub title: String,
    pub maximum: String,
    pub created_at_text: String,
    pub duration_minutes: String,
    pub status_label: String,
}

pub fn to_adaptive_identity_target_view(item: &AdaptiveIdentityTarget) -> AdaptiveIdentityTargetView {
    AdaptiveIdentityTargetView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        ends_at: item.ends_at.to_string(),
        capacity: item.capacity.to_string(),
        segment: text::truncate(&item.segment, 64),
        title: text::truncate(&item.title, 40),
        maximum: format!("{:.2}", item.maximum),
        created_at_text: item.created_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_identity_target_views(items: &[AdaptiveIdentityTarget]) -> Vec<AdaptiveIdentityTargetView> {
    let mut views: Vec<AdaptiveIdentityTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_identity_target_view(item));
    }
    views
}

pub fn sum_adaptive_identity_target_view(view: &AdaptiveIdentityTargetView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_capacity as f64) + parsed_maximum)
}
