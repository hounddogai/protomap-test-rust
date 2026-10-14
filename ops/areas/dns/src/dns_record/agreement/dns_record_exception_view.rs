use super::*;

use acme_kit::text;

/// How screens and exports show a dns record exception.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordExceptionView {
    pub code: String,
    pub title: String,
    pub minimum: String,
    pub segment_text: String,
    pub revision: String,
    pub expires_at: String,
    pub channel_text: String,
    pub rotation_days: String,
    pub ends_at_text: String,
    pub status_label: String,
}

pub fn to_dns_record_exception_view(item: &DnsRecordException) -> DnsRecordExceptionView {
    DnsRecordExceptionView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        minimum: format!("{:.2}", item.minimum),
        segment_text: text::truncate(&item.segment, 40),
        revision: item.revision.to_string(),
        expires_at: item.expires_at.to_string(),
        channel_text: item.channel.clone(),
        rotation_days: item.rotation_days.to_string(),
        ends_at_text: item.ends_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dns_record_exception_views(items: &[DnsRecordException]) -> Vec<DnsRecordExceptionView> {
    let mut views: Vec<DnsRecordExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_dns_record_exception_view(item));
    }
    views
}

pub fn sum_dns_record_exception_view(view: &DnsRecordExceptionView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_minimum + (parsed_revision as f64) + (parsed_expires_at as f64))
}
