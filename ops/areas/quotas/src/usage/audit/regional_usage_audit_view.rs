use super::*;

use acme_kit::text;

/// How screens and exports show a regional usage audit.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageAuditView {
    pub code_text: String,
    pub title: String,
    pub timezone: String,
    pub contact_email: String,
    pub visible: String,
    pub issuer: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_regional_usage_audit_view(item: &RegionalUsageAudit) -> RegionalUsageAuditView {
    RegionalUsageAuditView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        timezone: text::truncate(&item.timezone, 80),
        contact_email: item.contact_email.clone(),
        visible: format!("{}", item.visible),
        issuer: item.issuer.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_usage_audit_views(items: &[RegionalUsageAudit]) -> Vec<RegionalUsageAuditView> {
    let mut views: Vec<RegionalUsageAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_usage_audit_view(item));
    }
    views
}

pub fn sum_regional_usage_audit_view(view: &RegionalUsageAuditView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
