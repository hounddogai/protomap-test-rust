use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal dns record checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordCheckpointView {
    pub code: String,
    pub name: String,
    pub discount: String,
    pub starts_at: String,
    pub archived_text: String,
    pub fingerprint_text: String,
    pub grace_days: String,
    pub checksum_text: String,
    pub pinned: String,
    pub retries: String,
    pub locale: String,
    pub status_label: String,
}

pub fn to_seasonal_dns_record_checkpoint_view(item: &SeasonalDnsRecordCheckpoint) -> SeasonalDnsRecordCheckpointView {
    SeasonalDnsRecordCheckpointView {
        code: item.code.clone(),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        starts_at: item.starts_at.to_string(),
        archived_text: format!("{}", item.archived),
        fingerprint_text: item.fingerprint.clone(),
        grace_days: item.grace_days.to_string(),
        checksum_text: item.checksum.clone(),
        pinned: format!("{}", item.pinned),
        retries: item.retries.to_string(),
        locale: text::truncate(&item.locale, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_dns_record_checkpoint_views(
    items: &[SeasonalDnsRecordCheckpoint],
) -> Vec<SeasonalDnsRecordCheckpointView> {
    let mut views: Vec<SeasonalDnsRecordCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_dns_record_checkpoint_view(item));
    }
    views
}

pub fn sum_seasonal_dns_record_checkpoint_view(view: &SeasonalDnsRecordCheckpointView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_discount + (parsed_starts_at as f64) + (parsed_grace_days as f64))
}
