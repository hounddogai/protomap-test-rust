use super::*;

use acme_kit::text;

/// How screens and exports show a pending silence notice.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceNoticeView {
    pub code: String,
    pub name: String,
    pub sequence: String,
    pub source: String,
    pub locale: String,
    pub rate: String,
    pub key_id: String,
    pub currency: String,
    pub status_label: String,
}

pub fn to_pending_silence_notice_view(item: &PendingSilenceNotice) -> PendingSilenceNoticeView {
    PendingSilenceNoticeView {
        code: item.code.clone(),
        name: item.name.clone(),
        sequence: item.sequence.to_string(),
        source: text::truncate(&item.source, 40),
        locale: item.locale.clone(),
        rate: format!("{:.2}", item.rate),
        key_id: item.key_id.clone(),
        currency: item.currency.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_silence_notice_views(items: &[PendingSilenceNotice]) -> Vec<PendingSilenceNoticeView> {
    let mut views: Vec<PendingSilenceNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_silence_notice_view(item));
    }
    views
}

pub fn sum_pending_silence_notice_view(view: &PendingSilenceNoticeView) -> Result<f64, String> {
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_sequence as f64 + parsed_rate)
}
