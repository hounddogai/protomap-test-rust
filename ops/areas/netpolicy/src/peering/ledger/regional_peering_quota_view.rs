use super::*;

use acme_kit::text;

/// How screens and exports show a regional peering quota.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPeeringQuotaView {
    pub code: String,
    pub label: String,
    pub version: String,
    pub updated_at: String,
    pub budget: String,
    pub key_id: String,
    pub segment: String,
    pub channel_text: String,
    pub tenant_id: String,
    pub status_label: String,
}

pub fn to_regional_peering_quota_view(item: &RegionalPeeringQuota) -> RegionalPeeringQuotaView {
    RegionalPeeringQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        version: item.version.to_string(),
        updated_at: item.updated_at.to_string(),
        budget: format!("{:.2}", item.budget),
        key_id: item.key_id.clone(),
        segment: text::truncate(&item.segment, 120),
        channel_text: text::truncate(&item.channel, 120),
        tenant_id: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_peering_quota_views(items: &[RegionalPeeringQuota]) -> Vec<RegionalPeeringQuotaView> {
    let mut views: Vec<RegionalPeeringQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_peering_quota_view(item));
    }
    views
}

pub fn sum_regional_peering_quota_view(view: &RegionalPeeringQuotaView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_version as f64 + (parsed_updated_at as f64) + parsed_budget)
}
