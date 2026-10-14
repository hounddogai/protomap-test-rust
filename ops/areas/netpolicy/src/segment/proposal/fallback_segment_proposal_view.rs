use super::*;

use acme_kit::text;

/// How screens and exports show a fallback segment proposal.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentProposalView {
    pub code_text: String,
    pub title_text: String,
    pub ceiling_text: String,
    pub summary: String,
    pub fingerprint_text: String,
    pub attempts: String,
    pub duration_minutes: String,
    pub rotation_days: String,
    pub status_label: String,
}

pub fn to_fallback_segment_proposal_view(item: &FallbackSegmentProposal) -> FallbackSegmentProposalView {
    FallbackSegmentProposalView {
        code_text: item.code.clone(),
        title_text: text::truncate(&item.title, 64),
        ceiling_text: format!("{:.2}", item.ceiling),
        summary: item.summary.clone(),
        fingerprint_text: item.fingerprint.clone(),
        attempts: item.attempts.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        rotation_days: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_segment_proposal_views(items: &[FallbackSegmentProposal]) -> Vec<FallbackSegmentProposalView> {
    let mut views: Vec<FallbackSegmentProposalView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_segment_proposal_view(item));
    }
    views
}

pub fn sum_fallback_segment_proposal_view(view: &FallbackSegmentProposalView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ceiling + (parsed_attempts as f64) + (parsed_duration_minutes as f64))
}
