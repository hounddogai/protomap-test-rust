use super::*;

use acme_kit::text;

/// How screens and exports show a global evidence exception.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEvidenceExceptionView {
    pub code: String,
    pub title: String,
    pub label: String,
    pub contact_email_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_global_evidence_exception_view(item: &GlobalEvidenceException) -> GlobalEvidenceExceptionView {
    GlobalEvidenceExceptionView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        label: text::truncate(&item.label, 120),
        contact_email_text: text::truncate(&item.contact_email, 40),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_evidence_exception_views(items: &[GlobalEvidenceException]) -> Vec<GlobalEvidenceExceptionView> {
    let mut views: Vec<GlobalEvidenceExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_evidence_exception_view(item));
    }
    views
}

pub fn sum_global_evidence_exception_view(view: &GlobalEvidenceExceptionView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
