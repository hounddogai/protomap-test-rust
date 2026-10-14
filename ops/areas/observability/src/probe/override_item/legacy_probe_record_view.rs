use super::*;

use acme_kit::text;

/// How screens and exports show a legacy probe record.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyProbeRecordView {
    pub code_text: String,
    pub name: String,
    pub fingerprint: String,
    pub locale_text: String,
    pub ttl_seconds_text: String,
    pub status_label: String,
}

pub fn to_legacy_probe_record_view(item: &LegacyProbeRecord) -> LegacyProbeRecordView {
    LegacyProbeRecordView {
        code_text: text::truncate(&item.code, 80),
        name: item.name.clone(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        locale_text: item.locale.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_probe_record_views(items: &[LegacyProbeRecord]) -> Vec<LegacyProbeRecordView> {
    let mut views: Vec<LegacyProbeRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_probe_record_view(item));
    }
    views
}

pub fn sum_legacy_probe_record_view(view: &LegacyProbeRecordView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}
