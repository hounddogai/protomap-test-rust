use super::*;

use acme_kit::text;

/// How screens and exports show a rolling grant target.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantTargetView {
    pub code_text: String,
    pub name: String,
    pub expires_at: String,
    pub principal_text: String,
    pub checksum: String,
    pub attempts_text: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_rolling_grant_target_view(item: &RollingGrantTarget) -> RollingGrantTargetView {
    RollingGrantTargetView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        expires_at: item.expires_at.to_string(),
        principal_text: text::truncate(&item.principal, 40),
        checksum: text::truncate(&item.checksum, 80),
        attempts_text: item.attempts.to_string(),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_grant_target_views(items: &[RollingGrantTarget]) -> Vec<RollingGrantTargetView> {
    let mut views: Vec<RollingGrantTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_grant_target_view(item));
    }
    views
}

pub fn sum_rolling_grant_target_view(view: &RollingGrantTargetView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_attempts as f64))
}
