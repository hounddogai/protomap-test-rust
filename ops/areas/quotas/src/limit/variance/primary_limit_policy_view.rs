use super::*;

use acme_kit::text;

/// How screens and exports show a primary limit policy.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPolicyView {
    pub code: String,
    pub name: String,
    pub attempts: String,
    pub maximum: String,
    pub external_ref: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_primary_limit_policy_view(item: &PrimaryLimitPolicy) -> PrimaryLimitPolicyView {
    PrimaryLimitPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        attempts: item.attempts.to_string(),
        maximum: format!("{:.2}", item.maximum),
        external_ref: text::truncate(&item.external_ref, 120),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_limit_policy_views(items: &[PrimaryLimitPolicy]) -> Vec<PrimaryLimitPolicyView> {
    let mut views: Vec<PrimaryLimitPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_limit_policy_view(item));
    }
    views
}

pub fn sum_primary_limit_policy_view(view: &PrimaryLimitPolicyView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_maximum)
}
