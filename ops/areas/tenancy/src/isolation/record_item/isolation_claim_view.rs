use super::*;

use acme_kit::text;

/// How screens and exports show a isolation claim.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationClaimView {
    pub code_text: String,
    pub title: String,
    pub rotation_days_text: String,
    pub port: String,
    pub cipher: String,
    pub approved_text: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_isolation_claim_view(item: &IsolationClaim) -> IsolationClaimView {
    IsolationClaimView {
        code_text: text::truncate(&item.code, 120),
        title: text::truncate(&item.title, 40),
        rotation_days_text: item.rotation_days.to_string(),
        port: item.port.to_string(),
        cipher: item.cipher.clone(),
        approved_text: format!("{}", item.approved),
        scope_text: text::truncate(&item.scope, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_claim_views(items: &[IsolationClaim]) -> Vec<IsolationClaimView> {
    let mut views: Vec<IsolationClaimView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_claim_view(item));
    }
    views
}

pub fn sum_isolation_claim_view(view: &IsolationClaimView) -> Result<f64, String> {
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_rotation_days as f64 + (parsed_port as f64))
}
