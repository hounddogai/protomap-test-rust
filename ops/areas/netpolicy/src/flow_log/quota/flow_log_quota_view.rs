use super::*;

use acme_kit::text;

/// How screens and exports show a flow log quota.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogQuotaView {
    pub code: String,
    pub label_text: String,
    pub score: String,
    pub updated_at: String,
    pub attempts: String,
    pub burst: String,
    pub region: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_flow_log_quota_view(item: &FlowLogQuota) -> FlowLogQuotaView {
    FlowLogQuotaView {
        code: text::truncate(&item.code, 40),
        label_text: text::truncate(&item.label, 80),
        score: format!("{:.2}", item.score),
        updated_at: item.updated_at.to_string(),
        attempts: item.attempts.to_string(),
        burst: item.burst.to_string(),
        region: item.region.clone(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_quota_views(items: &[FlowLogQuota]) -> Vec<FlowLogQuotaView> {
    let mut views: Vec<FlowLogQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_quota_view(item));
    }
    views
}

pub fn sum_flow_log_quota_view(view: &FlowLogQuotaView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_score + (parsed_updated_at as f64) + (parsed_attempts as f64))
}
