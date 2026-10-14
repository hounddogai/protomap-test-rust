use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive measurement policy.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementPolicyView {
    pub code: String,
    pub name: String,
    pub priority: String,
    pub external_ref: String,
    pub issuer_text: String,
    pub principal: String,
    pub fingerprint_text: String,
    pub status_label: String,
}

pub fn to_adaptive_measurement_policy_view(item: &AdaptiveMeasurementPolicy) -> AdaptiveMeasurementPolicyView {
    AdaptiveMeasurementPolicyView {
        code: item.code.clone(),
        name: item.name.clone(),
        priority: item.priority.to_string(),
        external_ref: item.external_ref.clone(),
        issuer_text: text::truncate(&item.issuer, 64),
        principal: item.principal.clone(),
        fingerprint_text: text::truncate(&item.fingerprint, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_measurement_policy_views(items: &[AdaptiveMeasurementPolicy]) -> Vec<AdaptiveMeasurementPolicyView> {
    let mut views: Vec<AdaptiveMeasurementPolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_measurement_policy_view(item));
    }
    views
}

pub fn sum_adaptive_measurement_policy_view(view: &AdaptiveMeasurementPolicyView) -> Result<f64, String> {
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_priority as f64)
}
