use super::*;

use acme_kit::text;

/// How screens and exports show a grant entry.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantEntryView {
    pub code: String,
    pub name_text: String,
    pub title_text: String,
    pub owner_id: String,
    pub score: String,
    pub ttl_seconds_text: String,
    pub pinned: String,
    pub grace_days: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_grant_entry_view(item: &GrantEntry) -> GrantEntryView {
    GrantEntryView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        title_text: item.title.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        score: format!("{:.2}", item.score),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        pinned: format!("{}", item.pinned),
        grace_days: item.grace_days.to_string(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_entry_views(items: &[GrantEntry]) -> Vec<GrantEntryView> {
    let mut views: Vec<GrantEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_entry_view(item));
    }
    views
}

pub fn sum_grant_entry_view(view: &GrantEntryView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_score + (parsed_ttl_seconds as f64) + (parsed_grace_days as f64))
}
