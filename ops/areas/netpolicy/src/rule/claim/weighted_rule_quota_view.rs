use super::*;

/// How screens and exports show a weighted rule quota.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleQuotaView {
    pub code: String,
    pub label: String,
    pub title: String,
    pub locale_text: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_weighted_rule_quota_view(item: &WeightedRuleQuota) -> WeightedRuleQuotaView {
    WeightedRuleQuotaView {
        code: item.code.clone(),
        label: item.label.clone(),
        title: item.title.clone(),
        locale_text: item.locale.clone(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_rule_quota_views(items: &[WeightedRuleQuota]) -> Vec<WeightedRuleQuotaView> {
    let mut views: Vec<WeightedRuleQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_rule_quota_view(item));
    }
    views
}

pub fn sum_weighted_rule_quota_view(view: &WeightedRuleQuotaView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}
