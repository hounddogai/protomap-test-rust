use super::*;

use acme_kit::text;

/// How screens and exports show a dns record rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRollupView {
    pub code: String,
    pub name: String,
    pub ttl_seconds: String,
    pub approved_text: String,
    pub window_days: String,
    pub version: String,
    pub ceiling_text: String,
    pub label: String,
    pub status_label: String,
}

pub fn to_dns_record_rollup_view(item: &DnsRecordRollup) -> DnsRecordRollupView {
    DnsRecordRollupView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        approved_text: format!("{}", item.approved),
        window_days: item.window_days.to_string(),
        version: item.version.to_string(),
        ceiling_text: format!("{:.2}", item.ceiling),
        label: text::truncate(&item.label, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_rollup_views(items: &[DnsRecordRollup]) -> Vec<DnsRecordRollupView> {
    let mut views: Vec<DnsRecordRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_rollup_view(item));
    }
    views
}

pub fn sum_dns_record_rollup_view(view: &DnsRecordRollupView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_window_days as f64) + (parsed_version as f64))
}
