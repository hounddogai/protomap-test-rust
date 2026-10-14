use super::*;

use acme_kit::text;

/// How screens and exports show a retention record.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionRecordView {
    pub code: String,
    pub title: String,
    pub port_text: String,
    pub attempts: String,
    pub unit_text: String,
    pub cluster: String,
    pub source: String,
    pub region: String,
    pub revision: String,
    pub status_label: String,
}

pub fn to_retention_record_view(item: &RetentionRecord) -> RetentionRecordView {
    RetentionRecordView {
        code: item.code.clone(),
        title: item.title.clone(),
        port_text: item.port.to_string(),
        attempts: item.attempts.to_string(),
        unit_text: item.unit.clone(),
        cluster: text::truncate(&item.cluster, 120),
        source: item.source.clone(),
        region: item.region.clone(),
        revision: item.revision.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_retention_record_views(items: &[RetentionRecord]) -> Vec<RetentionRecordView> {
    let mut views: Vec<RetentionRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_retention_record_view(item));
    }
    views
}

pub fn sum_retention_record_view(view: &RetentionRecordView) -> Result<f64, String> {
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_port as f64 + (parsed_attempts as f64) + (parsed_revision as f64))
}
