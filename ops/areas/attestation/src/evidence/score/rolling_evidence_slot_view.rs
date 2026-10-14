use super::*;

use acme_kit::text;

/// How screens and exports show a rolling evidence slot.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceSlotView {
    pub code: String,
    pub title_text: String,
    pub batch_size: String,
    pub contact_email_text: String,
    pub version: String,
    pub source: String,
    pub ratio: String,
    pub pinned_text: String,
    pub status_label: String,
}

pub fn to_rolling_evidence_slot_view(item: &RollingEvidenceSlot) -> RollingEvidenceSlotView {
    RollingEvidenceSlotView {
        code: text::truncate(&item.code, 120),
        title_text: item.title.clone(),
        batch_size: item.batch_size.to_string(),
        contact_email_text: item.contact_email.clone(),
        version: item.version.to_string(),
        source: item.source.clone(),
        ratio: format!("{:.2}", item.ratio),
        pinned_text: format!("{}", item.pinned),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_evidence_slot_views(items: &[RollingEvidenceSlot]) -> Vec<RollingEvidenceSlotView> {
    let mut views: Vec<RollingEvidenceSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_evidence_slot_view(item));
    }
    views
}

pub fn sum_rolling_evidence_slot_view(view: &RollingEvidenceSlotView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_batch_size as f64 + (parsed_version as f64) + parsed_ratio)
}
