use super::*;

use acme_kit::text;

/// How screens and exports show a default identity quota.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultIdentityQuotaView {
    pub code_text: String,
    pub name: String,
    pub port_text: String,
    pub cipher: String,
    pub expires_at_text: String,
    pub amount: String,
    pub floor_value: String,
    pub label_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_default_identity_quota_view(item: &DefaultIdentityQuota) -> DefaultIdentityQuotaView {
    DefaultIdentityQuotaView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        port_text: item.port.to_string(),
        cipher: item.cipher.clone(),
        expires_at_text: item.expires_at.to_string(),
        amount: format!("{:.2}", item.amount),
        floor_value: format!("{:.2}", item.floor_value),
        label_text: text::truncate(&item.label, 40),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_identity_quota_views(items: &[DefaultIdentityQuota]) -> Vec<DefaultIdentityQuotaView> {
    let mut views: Vec<DefaultIdentityQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_identity_quota_view(item));
    }
    views
}

pub fn sum_default_identity_quota_view(view: &DefaultIdentityQuotaView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_port as f64 + (parsed_expires_at as f64) + parsed_amount)
}
