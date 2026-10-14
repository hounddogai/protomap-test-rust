use super::*;

use acme_kit::text;

/// How screens and exports show a manual tenant rule.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantRuleView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub discount: String,
    pub contact_email: String,
    pub cipher: String,
    pub rate: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_manual_tenant_rule_view(item: &ManualTenantRule) -> ManualTenantRuleView {
    ManualTenantRuleView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        quantity: item.quantity.to_string(),
        discount: format!("{:.2}", item.discount),
        contact_email: text::truncate(&item.contact_email, 64),
        cipher: text::truncate(&item.cipher, 120),
        rate: format!("{:.2}", item.rate),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_manual_tenant_rule_views(items: &[ManualTenantRule]) -> Vec<ManualTenantRuleView> {
    let mut views: Vec<ManualTenantRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_manual_tenant_rule_view(item));
    }
    views
}

pub fn sum_manual_tenant_rule_view(view: &ManualTenantRuleView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_discount + parsed_rate)
}
