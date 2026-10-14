use super::*;

use acme_kit::text;

/// How screens and exports show a batched identity target.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTargetView {
    pub code: String,
    pub name_text: String,
    pub category: String,
    pub budget: String,
    pub cost: String,
    pub sequence_text: String,
    pub fingerprint: String,
    pub external_ref: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_batched_identity_target_view(item: &BatchedIdentityTarget) -> BatchedIdentityTargetView {
    BatchedIdentityTargetView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        category: text::truncate(&item.category, 40),
        budget: format!("{:.2}", item.budget),
        cost: format!("{:.2}", item.cost),
        sequence_text: item.sequence.to_string(),
        fingerprint: text::truncate(&item.fingerprint, 120),
        external_ref: item.external_ref.clone(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_identity_target_views(items: &[BatchedIdentityTarget]) -> Vec<BatchedIdentityTargetView> {
    let mut views: Vec<BatchedIdentityTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_identity_target_view(item));
    }
    views
}

pub fn sum_batched_identity_target_view(view: &BatchedIdentityTargetView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_budget + parsed_cost + (parsed_sequence as f64))
}
