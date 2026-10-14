use super::*;

use acme_kit::text;

/// How screens and exports show a weighted federation cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationCycleView {
    pub code_text: String,
    pub title: String,
    pub segment: String,
    pub revision: String,
    pub unit_text: String,
    pub source: String,
    pub notes: String,
    pub starts_at: String,
    pub archived: String,
    pub score: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_weighted_federation_cycle_view(item: &WeightedFederationCycle) -> WeightedFederationCycleView {
    WeightedFederationCycleView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        segment: item.segment.clone(),
        revision: item.revision.to_string(),
        unit_text: item.unit.clone(),
        source: text::truncate(&item.source, 64),
        notes: item.notes.clone(),
        starts_at: item.starts_at.to_string(),
        archived: format!("{}", item.archived),
        score: format!("{:.2}", item.score),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_federation_cycle_views(items: &[WeightedFederationCycle]) -> Vec<WeightedFederationCycleView> {
    let mut views: Vec<WeightedFederationCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_federation_cycle_view(item));
    }
    views
}

pub fn sum_weighted_federation_cycle_view(view: &WeightedFederationCycleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_starts_at as f64) + parsed_score)
}
