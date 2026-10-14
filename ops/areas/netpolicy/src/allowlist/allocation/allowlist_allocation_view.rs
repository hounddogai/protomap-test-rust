use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAllocationView {
    pub code_text: String,
    pub label_text: String,
    pub fingerprint: String,
    pub retries: String,
    pub host: String,
    pub expires_at: String,
    pub source_text: String,
    pub revision: String,
    pub discount_text: String,
    pub status_label: String,
}

pub fn to_allowlist_allocation_view(item: &AllowlistAllocation) -> AllowlistAllocationView {
    AllowlistAllocationView {
        code_text: item.code.clone(),
        label_text: text::truncate(&item.label, 64),
        fingerprint: item.fingerprint.clone(),
        retries: item.retries.to_string(),
        host: text::truncate(&item.host, 120),
        expires_at: item.expires_at.to_string(),
        source_text: text::truncate(&item.source, 80),
        revision: item.revision.to_string(),
        discount_text: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_allocation_views(items: &[AllowlistAllocation]) -> Vec<AllowlistAllocationView> {
    let mut views: Vec<AllowlistAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_allocation_view(item));
    }
    views
}

pub fn sum_allowlist_allocation_view(view: &AllowlistAllocationView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_expires_at as f64) + (parsed_revision as f64))
}
