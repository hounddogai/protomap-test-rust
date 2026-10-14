use super::*;

use acme_kit::text;

/// How screens and exports show a credential notice.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialNoticeView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub threshold: String,
    pub retries: String,
    pub segment: String,
    pub duration_minutes: String,
    pub channel: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_credential_notice_view(item: &CredentialNotice) -> CredentialNoticeView {
    CredentialNoticeView {
        code: item.code.clone(),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        threshold: format!("{:.2}", item.threshold),
        retries: item.retries.to_string(),
        segment: text::truncate(&item.segment, 120),
        duration_minutes: item.duration_minutes.to_string(),
        channel: item.channel.clone(),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_notice_views(items: &[CredentialNotice]) -> Vec<CredentialNoticeView> {
    let mut views: Vec<CredentialNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_notice_view(item));
    }
    views
}

pub fn sum_credential_notice_view(view: &CredentialNoticeView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_threshold + (parsed_retries as f64))
}
