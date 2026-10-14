use super::*;

use acme_kit::text;

/// How screens and exports show a canary quota.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryQuotaView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub locale_text: String,
    pub quota_units: String,
    pub cipher: String,
    pub capacity: String,
    pub currency: String,
    pub visible: String,
    pub budget: String,
    pub phone: String,
    pub status_label: String,
}

pub fn to_canary_quota_view(item: &CanaryQuota) -> CanaryQuotaView {
    CanaryQuotaView {
        code: text::truncate(&item.code, 40),
        name: item.name.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        locale_text: text::truncate(&item.locale, 64),
        quota_units: item.quota_units.to_string(),
        cipher: item.cipher.clone(),
        capacity: item.capacity.to_string(),
        currency: text::truncate(&item.currency, 80),
        visible: format!("{}", item.visible),
        budget: format!("{:.2}", item.budget),
        phone: item.phone.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_canary_quota_views(items: &[CanaryQuota]) -> Vec<CanaryQuotaView> {
    let mut views: Vec<CanaryQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_canary_quota_view(item));
    }
    views
}

pub fn sum_canary_quota_view(view: &CanaryQuotaView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_capacity as f64) + parsed_budget)
}
