use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled evidence tier.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEvidenceTierView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub ends_at: String,
    pub floor_value_text: String,
    pub weight: String,
    pub archived: String,
    pub capacity: String,
    pub quota_units: String,
    pub label: String,
    pub ceiling: String,
    pub status_label: String,
}

pub fn to_scheduled_evidence_tier_view(item: &ScheduledEvidenceTier) -> ScheduledEvidenceTierView {
    ScheduledEvidenceTierView {
        code: item.code.clone(),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        ends_at: item.ends_at.to_string(),
        floor_value_text: format!("{:.2}", item.floor_value),
        weight: format!("{:.2}", item.weight),
        archived: format!("{}", item.archived),
        capacity: item.capacity.to_string(),
        quota_units: item.quota_units.to_string(),
        label: text::truncate(&item.label, 40),
        ceiling: format!("{:.2}", item.ceiling),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_evidence_tier_views(items: &[ScheduledEvidenceTier]) -> Vec<ScheduledEvidenceTierView> {
    let mut views: Vec<ScheduledEvidenceTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_evidence_tier_view(item));
    }
    views
}

pub fn sum_scheduled_evidence_tier_view(view: &ScheduledEvidenceTierView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_ends_at as f64) + parsed_floor_value)
}
