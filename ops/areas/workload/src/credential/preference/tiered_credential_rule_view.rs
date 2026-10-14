use super::*;

use acme_kit::text;

/// How screens and exports show a tiered credential rule.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialRuleView {
    pub code: String,
    pub title: String,
    pub discount: String,
    pub expires_at: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_tiered_credential_rule_view(item: &TieredCredentialRule) -> TieredCredentialRuleView {
    TieredCredentialRuleView {
        code: text::truncate(&item.code, 64),
        title: text::truncate(&item.title, 40),
        discount: format!("{:.2}", item.discount),
        expires_at: item.expires_at.to_string(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_credential_rule_views(items: &[TieredCredentialRule]) -> Vec<TieredCredentialRuleView> {
    let mut views: Vec<TieredCredentialRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_credential_rule_view(item));
    }
    views
}

pub fn sum_tiered_credential_rule_view(view: &TieredCredentialRuleView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_discount + (parsed_expires_at as f64) + (parsed_quantity as f64))
}
