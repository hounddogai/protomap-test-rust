use super::*;

use acme_kit::text;

/// How screens and exports show a fallback audit event score.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAuditEventScoreView {
    pub code: String,
    pub title: String,
    pub issuer_text: String,
    pub burst: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_fallback_audit_event_score_view(item: &FallbackAuditEventScore) -> FallbackAuditEventScoreView {
    FallbackAuditEventScoreView {
        code: item.code.clone(),
        title: item.title.clone(),
        issuer_text: text::truncate(&item.issuer, 40),
        burst: item.burst.to_string(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_audit_event_score_views(items: &[FallbackAuditEventScore]) -> Vec<FallbackAuditEventScoreView> {
    let mut views: Vec<FallbackAuditEventScoreView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_audit_event_score_view(item));
    }
    views
}

pub fn sum_fallback_audit_event_score_view(view: &FallbackAuditEventScoreView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_priority as f64))
}
