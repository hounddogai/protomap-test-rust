use super::*;

use acme_kit::text;

/// How screens and exports show a rollback segment.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSegmentView {
    pub code_text: String,
    pub name: String,
    pub issuer: String,
    pub budget_text: String,
    pub fingerprint: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_rollback_segment_view(item: &RollbackSegment) -> RollbackSegmentView {
    RollbackSegmentView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        issuer: item.issuer.clone(),
        budget_text: format!("{:.2}", item.budget),
        fingerprint: text::truncate(&item.fingerprint, 80),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rollback_segment_views(items: &[RollbackSegment]) -> Vec<RollbackSegmentView> {
    let mut views: Vec<RollbackSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rollback_segment_view(item));
    }
    views
}

pub fn sum_rollback_segment_view(view: &RollbackSegmentView) -> Result<f64, String> {
    let parsed_budget = view
        .budget_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_budget)
}
