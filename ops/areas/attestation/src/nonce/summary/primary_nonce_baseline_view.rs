use super::*;

use acme_kit::text;

/// How screens and exports show a primary nonce baseline.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryNonceBaselineView {
    pub code: String,
    pub label_text: String,
    pub created_at_text: String,
    pub grace_days_text: String,
    pub phone: String,
    pub capacity_text: String,
    pub budget: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_primary_nonce_baseline_view(item: &PrimaryNonceBaseline) -> PrimaryNonceBaselineView {
    PrimaryNonceBaselineView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        created_at_text: item.created_at.to_string(),
        grace_days_text: item.grace_days.to_string(),
        phone: text::truncate(&item.phone, 120),
        capacity_text: item.capacity.to_string(),
        budget: format!("{:.2}", item.budget),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_nonce_baseline_views(items: &[PrimaryNonceBaseline]) -> Vec<PrimaryNonceBaselineView> {
    let mut views: Vec<PrimaryNonceBaselineView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_nonce_baseline_view(item));
    }
    views
}

pub fn sum_primary_nonce_baseline_view(view: &PrimaryNonceBaselineView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_grace_days as f64) + (parsed_capacity as f64))
}
