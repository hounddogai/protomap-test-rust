use super::*;

use acme_kit::text;

/// How screens and exports show a credential audit.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAuditView {
    pub code: String,
    pub title_text: String,
    pub locale_text: String,
    pub quota_units: String,
    pub category_text: String,
    pub margin_text: String,
    pub amount_text: String,
    pub notes_text: String,
    pub checksum_text: String,
    pub status_label: String,
}

pub fn to_credential_audit_view(item: &CredentialAudit) -> CredentialAuditView {
    CredentialAuditView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        locale_text: item.locale.clone(),
        quota_units: item.quota_units.to_string(),
        category_text: item.category.clone(),
        margin_text: format!("{:.2}", item.margin),
        amount_text: format!("{:.2}", item.amount),
        notes_text: text::truncate(&item.notes, 64),
        checksum_text: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_audit_views(items: &[CredentialAudit]) -> Vec<CredentialAuditView> {
    let mut views: Vec<CredentialAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_audit_view(item));
    }
    views
}

pub fn sum_credential_audit_view(view: &CredentialAuditView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_margin + parsed_amount)
}
