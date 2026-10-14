use super::*;

use acme_kit::text;

/// How screens and exports show a scheduled verifier summary.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledVerifierSummaryView {
    pub code: String,
    pub label: String,
    pub pinned: String,
    pub timezone: String,
    pub tenant_id_text: String,
    pub limit: String,
    pub notes: String,
    pub status_label: String,
}

pub fn to_scheduled_verifier_summary_view(item: &ScheduledVerifierSummary) -> ScheduledVerifierSummaryView {
    ScheduledVerifierSummaryView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        pinned: format!("{}", item.pinned),
        timezone: item.timezone.clone(),
        tenant_id_text: item.tenant_id.clone(),
        limit: item.limit.to_string(),
        notes: text::truncate(&item.notes, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_verifier_summary_views(items: &[ScheduledVerifierSummary]) -> Vec<ScheduledVerifierSummaryView> {
    let mut views: Vec<ScheduledVerifierSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_verifier_summary_view(item));
    }
    views
}

pub fn sum_scheduled_verifier_summary_view(view: &ScheduledVerifierSummaryView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}
