use super::*;

use acme_kit::text;

/// How screens and exports show a binding draft.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingDraftView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub cost: String,
    pub starts_at: String,
    pub category_text: String,
    pub weight: String,
    pub notes_text: String,
    pub burst: String,
    pub amount: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_binding_draft_view(item: &BindingDraft) -> BindingDraftView {
    BindingDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        cost: format!("{:.2}", item.cost),
        starts_at: item.starts_at.to_string(),
        category_text: item.category.clone(),
        weight: format!("{:.2}", item.weight),
        notes_text: text::truncate(&item.notes, 64),
        burst: item.burst.to_string(),
        amount: format!("{:.2}", item.amount),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_draft_views(items: &[BindingDraft]) -> Vec<BindingDraftView> {
    let mut views: Vec<BindingDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_draft_view(item));
    }
    views
}

pub fn sum_binding_draft_view(view: &BindingDraftView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_retries as f64 + parsed_cost + (parsed_starts_at as f64))
}
