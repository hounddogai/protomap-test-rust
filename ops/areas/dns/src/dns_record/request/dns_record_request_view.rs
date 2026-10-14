use super::*;

use acme_kit::text;

/// How screens and exports show a dns record request.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRequestView {
    pub code_text: String,
    pub label: String,
    pub channel_text: String,
    pub active: String,
    pub expires_at_text: String,
    pub threshold_text: String,
    pub duration_minutes_text: String,
    pub region: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_dns_record_request_view(item: &DnsRecordRequest) -> DnsRecordRequestView {
    DnsRecordRequestView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        channel_text: text::truncate(&item.channel, 40),
        active: format!("{}", item.active),
        expires_at_text: item.expires_at.to_string(),
        threshold_text: format!("{:.2}", item.threshold),
        duration_minutes_text: item.duration_minutes.to_string(),
        region: item.region.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_request_views(items: &[DnsRecordRequest]) -> Vec<DnsRecordRequestView> {
    let mut views: Vec<DnsRecordRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_request_view(item));
    }
    views
}

pub fn sum_dns_record_request_view(view: &DnsRecordRequestView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_expires_at as f64 + parsed_threshold + (parsed_duration_minutes as f64))
}
