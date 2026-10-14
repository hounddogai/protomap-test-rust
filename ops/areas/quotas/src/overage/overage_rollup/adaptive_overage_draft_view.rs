use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive overage draft.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveOverageDraftView {
    pub code_text: String,
    pub title_text: String,
    pub label: String,
    pub amount_text: String,
    pub batch_size: String,
    pub contact_email: String,
    pub pinned: String,
    pub status_label: String,
}

pub fn to_adaptive_overage_draft_view(item: &AdaptiveOverageDraft) -> AdaptiveOverageDraftView {
    AdaptiveOverageDraftView {
        code_text: item.code.clone(),
        title_text: item.title.clone(),
        label: text::truncate(&item.label, 120),
        amount_text: format!("{:.2}", item.amount),
        batch_size: item.batch_size.to_string(),
        contact_email: item.contact_email.clone(),
        pinned: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_overage_draft_views(items: &[AdaptiveOverageDraft]) -> Vec<AdaptiveOverageDraftView> {
    let mut views: Vec<AdaptiveOverageDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_overage_draft_view(item));
    }
    views
}

pub fn sum_adaptive_overage_draft_view(view: &AdaptiveOverageDraftView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_amount + (parsed_batch_size as f64))
}
