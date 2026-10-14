use super::*;

use acme_kit::text;

/// How screens and exports show a capped policy audit.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPolicyAuditView {
    pub code_text: String,
    pub label: String,
    pub created_at: String,
    pub source_text: String,
    pub notes: String,
    pub archived: String,
    pub status_label: String,
}

pub fn to_capped_policy_audit_view(item: &CappedPolicyAudit) -> CappedPolicyAuditView {
    CappedPolicyAuditView {
        code_text: text::truncate(&item.code, 40),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        source_text: item.source.clone(),
        notes: item.notes.clone(),
        archived: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_policy_audit_views(items: &[CappedPolicyAudit]) -> Vec<CappedPolicyAuditView> {
    let mut views: Vec<CappedPolicyAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_policy_audit_view(item));
    }
    views
}

pub fn sum_capped_policy_audit_view(view: &CappedPolicyAuditView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_created_at as f64)
}
