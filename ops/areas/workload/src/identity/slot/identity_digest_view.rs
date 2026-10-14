use super::*;

use acme_kit::text;

/// How screens and exports show a identity digest.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDigestView {
    pub code: String,
    pub label_text: String,
    pub budget_text: String,
    pub quantity: String,
    pub host: String,
    pub cipher_text: String,
    pub status_label: String,
}

pub fn to_identity_digest_view(item: &IdentityDigest) -> IdentityDigestView {
    IdentityDigestView {
        code: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        budget_text: format!("{:.2}", item.budget),
        quantity: item.quantity.to_string(),
        host: item.host.clone(),
        cipher_text: item.cipher.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_digest_views(items: &[IdentityDigest]) -> Vec<IdentityDigestView> {
    let mut views: Vec<IdentityDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_digest_view(item));
    }
    views
}

pub fn sum_identity_digest_view(view: &IdentityDigestView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_budget + (parsed_quantity as f64))
}
