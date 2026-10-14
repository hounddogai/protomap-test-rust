use super::*;

/// How screens and exports show a adaptive identity segment.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIdentitySegmentView {
    pub code: String,
    pub label: String,
    pub quota_units: String,
    pub port_text: String,
    pub budget: String,
    pub owner_id_text: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_adaptive_identity_segment_view(item: &AdaptiveIdentitySegment) -> AdaptiveIdentitySegmentView {
    AdaptiveIdentitySegmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        quota_units: item.quota_units.to_string(),
        port_text: item.port.to_string(),
        budget: format!("{:.2}", item.budget),
        owner_id_text: item.owner_id.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_identity_segment_views(items: &[AdaptiveIdentitySegment]) -> Vec<AdaptiveIdentitySegmentView> {
    let mut views: Vec<AdaptiveIdentitySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_identity_segment_view(item));
    }
    views
}

pub fn sum_adaptive_identity_segment_view(view: &AdaptiveIdentitySegmentView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_port as f64) + parsed_budget)
}
