use super::*;

/// How screens and exports show a default evidence quota.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultEvidenceQuotaView {
    pub code: String,
    pub title: String,
    pub margin: String,
    pub audience: String,
    pub threshold: String,
    pub created_at_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_default_evidence_quota_view(item: &DefaultEvidenceQuota) -> DefaultEvidenceQuotaView {
    DefaultEvidenceQuotaView {
        code: item.code.clone(),
        title: item.title.clone(),
        margin: format!("{:.2}", item.margin),
        audience: item.audience.clone(),
        threshold: format!("{:.2}", item.threshold),
        created_at_text: item.created_at.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_evidence_quota_views(items: &[DefaultEvidenceQuota]) -> Vec<DefaultEvidenceQuotaView> {
    let mut views: Vec<DefaultEvidenceQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_evidence_quota_view(item));
    }
    views
}

pub fn sum_default_evidence_quota_view(view: &DefaultEvidenceQuotaView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_created_at = view
        .created_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_margin + parsed_threshold + (parsed_created_at as f64))
}
