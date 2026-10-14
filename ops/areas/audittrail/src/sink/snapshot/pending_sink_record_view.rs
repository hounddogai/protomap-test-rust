use super::*;

use acme_kit::text;

/// How screens and exports show a pending sink record.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSinkRecordView {
    pub code_text: String,
    pub label: String,
    pub sequence: String,
    pub namespace: String,
    pub ttl_seconds_text: String,
    pub burst: String,
    pub starts_at: String,
    pub status_label: String,
}

pub fn to_pending_sink_record_view(item: &PendingSinkRecord) -> PendingSinkRecordView {
    PendingSinkRecordView {
        code_text: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 80),
        sequence: item.sequence.to_string(),
        namespace: text::truncate(&item.namespace, 80),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        burst: item.burst.to_string(),
        starts_at: item.starts_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_sink_record_views(items: &[PendingSinkRecord]) -> Vec<PendingSinkRecordView> {
    let mut views: Vec<PendingSinkRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_sink_record_view(item));
    }
    views
}

pub fn sum_pending_sink_record_view(view: &PendingSinkRecordView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_sequence as f64 + (parsed_ttl_seconds as f64) + (parsed_burst as f64))
}
