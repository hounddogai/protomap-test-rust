use super::*;

use acme_kit::text;

/// How screens and exports show a strict policy segment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictPolicySegmentView {
    pub code: String,
    pub label: String,
    pub country: String,
    pub updated_at: String,
    pub duration_minutes_text: String,
    pub host_text: String,
    pub cluster_text: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_strict_policy_segment_view(item: &StrictPolicySegment) -> StrictPolicySegmentView {
    StrictPolicySegmentView {
        code: item.code.clone(),
        label: item.label.clone(),
        country: text::truncate(&item.country, 80),
        updated_at: item.updated_at.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        host_text: item.host.clone(),
        cluster_text: item.cluster.clone(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_policy_segment_views(items: &[StrictPolicySegment]) -> Vec<StrictPolicySegmentView> {
    let mut views: Vec<StrictPolicySegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_policy_segment_view(item));
    }
    views
}

pub fn sum_strict_policy_segment_view(view: &StrictPolicySegmentView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_duration_minutes as f64) + (parsed_rotation_days as f64))
}
