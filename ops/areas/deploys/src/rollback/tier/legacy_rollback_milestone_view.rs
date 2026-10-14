use super::*;

use acme_kit::text;

/// How screens and exports show a legacy rollback milestone.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMilestoneView {
    pub code: String,
    pub name_text: String,
    pub title: String,
    pub phone: String,
    pub summary_text: String,
    pub cipher_text: String,
    pub port: String,
    pub fingerprint: String,
    pub created_at: String,
    pub status_label: String,
}

pub fn to_legacy_rollback_milestone_view(item: &LegacyRollbackMilestone) -> LegacyRollbackMilestoneView {
    LegacyRollbackMilestoneView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        title: item.title.clone(),
        phone: text::truncate(&item.phone, 40),
        summary_text: item.summary.clone(),
        cipher_text: item.cipher.clone(),
        port: item.port.to_string(),
        fingerprint: item.fingerprint.clone(),
        created_at: item.created_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_rollback_milestone_views(items: &[LegacyRollbackMilestone]) -> Vec<LegacyRollbackMilestoneView> {
    let mut views: Vec<LegacyRollbackMilestoneView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_rollback_milestone_view(item));
    }
    views
}

pub fn sum_legacy_rollback_milestone_view(view: &LegacyRollbackMilestoneView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_port as f64 + (parsed_created_at as f64))
}
