use super::*;

use acme_kit::text;

/// How screens and exports show a rule quota.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleQuotaView {
    pub code_text: String,
    pub name: String,
    pub description_text: String,
    pub country: String,
    pub locale: String,
    pub phone: String,
    pub namespace: String,
    pub cost: String,
    pub capacity: String,
    pub rate: String,
    pub updated_at: String,
    pub status_label: String,
}

pub fn to_rule_quota_view(item: &RuleQuota) -> RuleQuotaView {
    RuleQuotaView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        description_text: text::truncate(&item.description, 40),
        country: item.country.clone(),
        locale: item.locale.clone(),
        phone: item.phone.clone(),
        namespace: item.namespace.clone(),
        cost: format!("{:.2}", item.cost),
        capacity: item.capacity.to_string(),
        rate: format!("{:.2}", item.rate),
        updated_at: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_quota_views(items: &[RuleQuota]) -> Vec<RuleQuotaView> {
    let mut views: Vec<RuleQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_quota_view(item));
    }
    views
}

pub fn sum_rule_quota_view(view: &RuleQuotaView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_cost + (parsed_capacity as f64) + parsed_rate)
}
