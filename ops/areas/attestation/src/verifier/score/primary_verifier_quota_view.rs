use super::*;

use acme_kit::text;

/// How screens and exports show a primary verifier quota.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierQuotaView {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub quantity: String,
    pub channel: String,
    pub status_label: String,
}

pub fn to_primary_verifier_quota_view(item: &PrimaryVerifierQuota) -> PrimaryVerifierQuotaView {
    PrimaryVerifierQuotaView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 120),
        reviewer: item.reviewer.clone(),
        quantity: item.quantity.to_string(),
        channel: item.channel.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_verifier_quota_views(items: &[PrimaryVerifierQuota]) -> Vec<PrimaryVerifierQuotaView> {
    let mut views: Vec<PrimaryVerifierQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_verifier_quota_view(item));
    }
    views
}

pub fn sum_primary_verifier_quota_view(view: &PrimaryVerifierQuotaView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
