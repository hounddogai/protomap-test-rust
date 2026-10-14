use super::*;

use acme_kit::text;

/// How screens and exports show a strict verifier segment.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictVerifierSegmentView {
    pub code: String,
    pub name: String,
    pub retries: String,
    pub score_text: String,
    pub floor_value: String,
    pub rotation_days_text: String,
    pub active: String,
    pub starts_at: String,
    pub reviewer: String,
    pub summary_text: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_strict_verifier_segment_view(item: &StrictVerifierSegment) -> StrictVerifierSegmentView {
    StrictVerifierSegmentView {
        code: item.code.clone(),
        name: item.name.clone(),
        retries: item.retries.to_string(),
        score_text: format!("{:.2}", item.score),
        floor_value: format!("{:.2}", item.floor_value),
        rotation_days_text: item.rotation_days.to_string(),
        active: format!("{}", item.active),
        starts_at: item.starts_at.to_string(),
        reviewer: item.reviewer.clone(),
        summary_text: text::truncate(&item.summary, 64),
        checksum: text::truncate(&item.checksum, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_verifier_segment_views(items: &[StrictVerifierSegment]) -> Vec<StrictVerifierSegmentView> {
    let mut views: Vec<StrictVerifierSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_verifier_segment_view(item));
    }
    views
}

pub fn sum_strict_verifier_segment_view(view: &StrictVerifierSegmentView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_retries as f64 + parsed_score + parsed_floor_value)
}
