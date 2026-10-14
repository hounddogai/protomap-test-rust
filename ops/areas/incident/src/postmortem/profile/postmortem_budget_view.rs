use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem budget.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBudgetView {
    pub code_text: String,
    pub title_text: String,
    pub unit: String,
    pub namespace_text: String,
    pub segment_text: String,
    pub updated_at: String,
    pub contact_email: String,
    pub owner_id_text: String,
    pub cipher: String,
    pub duration_minutes: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_postmortem_budget_view(item: &PostmortemBudget) -> PostmortemBudgetView {
    PostmortemBudgetView {
        code_text: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        unit: item.unit.clone(),
        namespace_text: item.namespace.clone(),
        segment_text: item.segment.clone(),
        updated_at: item.updated_at.to_string(),
        contact_email: item.contact_email.clone(),
        owner_id_text: item.owner_id.clone(),
        cipher: item.cipher.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_budget_views(items: &[PostmortemBudget]) -> Vec<PostmortemBudgetView> {
    let mut views: Vec<PostmortemBudgetView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_budget_view(item));
    }
    views
}

pub fn sum_postmortem_budget_view(view: &PostmortemBudgetView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_updated_at as f64 + (parsed_duration_minutes as f64) + (parsed_quota_units as f64))
}
