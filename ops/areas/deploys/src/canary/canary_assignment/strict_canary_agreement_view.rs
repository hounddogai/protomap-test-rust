use super::*;

use acme_kit::text;

/// How screens and exports show a strict canary agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictCanaryAgreementView {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub key_id: String,
    pub updated_at: String,
    pub category: String,
    pub created_at: String,
    pub fingerprint: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_strict_canary_agreement_view(item: &StrictCanaryAgreement) -> StrictCanaryAgreementView {
    StrictCanaryAgreementView {
        code: item.code.clone(),
        name: item.name.clone(),
        summary: item.summary.clone(),
        key_id: text::truncate(&item.key_id, 80),
        updated_at: item.updated_at.to_string(),
        category: item.category.clone(),
        created_at: item.created_at.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 80),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_canary_agreement_views(items: &[StrictCanaryAgreement]) -> Vec<StrictCanaryAgreementView> {
    let mut views: Vec<StrictCanaryAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_canary_agreement_view(item));
    }
    views
}

pub fn sum_strict_canary_agreement_view(view: &StrictCanaryAgreementView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_created_at as f64))
}
