use super::*;

/// How screens and exports show a global release policy.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleasePolicyView {
    pub code: String,
    pub label_text: String,
    pub quantity_text: String,
    pub key_id_text: String,
    pub minimum: String,
    pub rate: String,
    pub host: String,
    pub attempts: String,
    pub region: String,
    pub channel_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_global_release_policy_view(item: &GlobalReleasePolicy) -> GlobalReleasePolicyView {
    GlobalReleasePolicyView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        quantity_text: item.quantity.to_string(),
        key_id_text: item.key_id.clone(),
        minimum: format!("{:.2}", item.minimum),
        rate: format!("{:.2}", item.rate),
        host: item.host.clone(),
        attempts: item.attempts.to_string(),
        region: item.region.clone(),
        channel_text: item.channel.clone(),
        issuer: item.issuer.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_release_policy_views(items: &[GlobalReleasePolicy]) -> Vec<GlobalReleasePolicyView> {
    let mut views: Vec<GlobalReleasePolicyView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_release_policy_view(item));
    }
    views
}

pub fn sum_global_release_policy_view(view: &GlobalReleasePolicyView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_minimum + parsed_rate)
}
