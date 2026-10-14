use super::*;

use acme_kit::text;

/// How screens and exports show a approval digest.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalDigestView {
    pub code: String,
    pub title_text: String,
    pub external_ref: String,
    pub attempts_text: String,
    pub rate: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_approval_digest_view(item: &ApprovalDigest) -> ApprovalDigestView {
    ApprovalDigestView {
        code: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        external_ref: text::truncate(&item.external_ref, 64),
        attempts_text: item.attempts.to_string(),
        rate: format!("{:.2}", item.rate),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_digest_views(items: &[ApprovalDigest]) -> Vec<ApprovalDigestView> {
    let mut views: Vec<ApprovalDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_digest_view(item));
    }
    views
}

pub fn sum_approval_digest_view(view: &ApprovalDigestView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_attempts as f64 + parsed_rate + (parsed_grace_days as f64))
}
