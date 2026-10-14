use super::*;

use acme_kit::text;

/// How screens and exports show a approval preference.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalPreferenceView {
    pub code: String,
    pub label_text: String,
    pub score_text: String,
    pub burst: String,
    pub approved: String,
    pub quantity_text: String,
    pub issuer: String,
    pub pinned: String,
    pub name: String,
    pub batch_size: String,
    pub status_label: String,
}

pub fn to_approval_preference_view(item: &ApprovalPreference) -> ApprovalPreferenceView {
    ApprovalPreferenceView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        score_text: format!("{:.2}", item.score),
        burst: item.burst.to_string(),
        approved: format!("{}", item.approved),
        quantity_text: item.quantity.to_string(),
        issuer: item.issuer.clone(),
        pinned: format!("{}", item.pinned),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_preference_views(items: &[ApprovalPreference]) -> Vec<ApprovalPreferenceView> {
    let mut views: Vec<ApprovalPreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_preference_view(item));
    }
    views
}

pub fn sum_approval_preference_view(view: &ApprovalPreferenceView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_score + (parsed_burst as f64) + (parsed_quantity as f64))
}
