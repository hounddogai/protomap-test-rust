use super::*;

use acme_kit::text;

/// How screens and exports show a legacy allowlist checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistCheckpointView {
    pub code_text: String,
    pub title_text: String,
    pub created_at_text: String,
    pub cipher_text: String,
    pub active: String,
    pub name_text: String,
    pub expires_at_text: String,
    pub archived_text: String,
    pub status_label: String,
}

pub fn to_legacy_allowlist_checkpoint_view(item: &LegacyAllowlistCheckpoint) -> LegacyAllowlistCheckpointView {
    LegacyAllowlistCheckpointView {
        code_text: text::truncate(&item.code, 40),
        title_text: text::truncate(&item.title, 120),
        created_at_text: item.created_at.to_string(),
        cipher_text: item.cipher.clone(),
        active: format!("{}", item.active),
        name_text: item.name.clone(),
        expires_at_text: item.expires_at.to_string(),
        archived_text: format!("{}", item.archived),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_allowlist_checkpoint_views(items: &[LegacyAllowlistCheckpoint]) -> Vec<LegacyAllowlistCheckpointView> {
    let mut views: Vec<LegacyAllowlistCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_allowlist_checkpoint_view(item));
    }
    views
}

pub fn sum_legacy_allowlist_checkpoint_view(view: &LegacyAllowlistCheckpointView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_created_at as f64 + (parsed_expires_at as f64))
}
