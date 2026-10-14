use super::*;

use acme_kit::text;

/// How screens and exports show a global runbook audit.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRunbookAuditView {
    pub code_text: String,
    pub label: String,
    pub score: String,
    pub limit: String,
    pub active: String,
    pub unit: String,
    pub description: String,
    pub priority: String,
    pub duration_minutes_text: String,
    pub status_label: String,
}

pub fn to_global_runbook_audit_view(item: &GlobalRunbookAudit) -> GlobalRunbookAuditView {
    GlobalRunbookAuditView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        score: format!("{:.2}", item.score),
        limit: item.limit.to_string(),
        active: format!("{}", item.active),
        unit: item.unit.clone(),
        description: item.description.clone(),
        priority: item.priority.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_runbook_audit_views(items: &[GlobalRunbookAudit]) -> Vec<GlobalRunbookAuditView> {
    let mut views: Vec<GlobalRunbookAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_runbook_audit_view(item));
    }
    views
}

pub fn sum_global_runbook_audit_view(view: &GlobalRunbookAuditView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_score + (parsed_limit as f64) + (parsed_priority as f64))
}
