use super::*;

use acme_kit::text;

/// How screens and exports show a dns record checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCheckpointView {
    pub code: String,
    pub title: String,
    pub ends_at: String,
    pub port: String,
    pub description_text: String,
    pub notes_text: String,
    pub checksum: String,
    pub created_at: String,
    pub duration_minutes: String,
    pub score_text: String,
    pub scope: String,
    pub status_label: String,
}

pub fn to_dns_record_checkpoint_view(item: &DnsRecordCheckpoint) -> DnsRecordCheckpointView {
    DnsRecordCheckpointView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        ends_at: item.ends_at.to_string(),
        port: item.port.to_string(),
        description_text: text::truncate(&item.description, 40),
        notes_text: item.notes.clone(),
        checksum: text::truncate(&item.checksum, 64),
        created_at: item.created_at.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        score_text: format!("{:.2}", item.score),
        scope: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_checkpoint_views(items: &[DnsRecordCheckpoint]) -> Vec<DnsRecordCheckpointView> {
    let mut views: Vec<DnsRecordCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_checkpoint_view(item));
    }
    views
}

pub fn sum_dns_record_checkpoint_view(view: &DnsRecordCheckpointView) -> Result<f64, String> {
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    Ok(parsed_ends_at as f64 + (parsed_port as f64) + (parsed_created_at as f64))
}
