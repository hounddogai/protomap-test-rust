use super::*;

/// How screens and exports show a regional grant quota.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalGrantQuotaView {
    pub code: String,
    pub name: String,
    pub minimum: String,
    pub port: String,
    pub attempts_text: String,
    pub weight: String,
    pub budget: String,
    pub starts_at_text: String,
    pub status_label: String,
}

pub fn to_regional_grant_quota_view(item: &RegionalGrantQuota) -> RegionalGrantQuotaView {
    RegionalGrantQuotaView {
        code: item.code.clone(),
        name: item.name.clone(),
        minimum: format!("{:.2}", item.minimum),
        port: item.port.to_string(),
        attempts_text: item.attempts.to_string(),
        weight: format!("{:.2}", item.weight),
        budget: format!("{:.2}", item.budget),
        starts_at_text: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_grant_quota_views(items: &[RegionalGrantQuota]) -> Vec<RegionalGrantQuotaView> {
    let mut views: Vec<RegionalGrantQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_grant_quota_view(item));
    }
    views
}

pub fn sum_regional_grant_quota_view(view: &RegionalGrantQuotaView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_minimum + (parsed_port as f64) + (parsed_attempts as f64))
}
