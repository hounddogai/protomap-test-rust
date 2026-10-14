use super::*;

use acme_kit::text;

/// How screens and exports show a redaction notice.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionNoticeView {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub key_id_text: String,
    pub source: String,
    pub country: String,
    pub title_text: String,
    pub label_text: String,
    pub revision: String,
    pub margin: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_redaction_notice_view(item: &RedactionNotice) -> RedactionNoticeView {
    RedactionNoticeView {
        code: item.code.clone(),
        name: item.name.clone(),
        fingerprint: item.fingerprint.clone(),
        key_id_text: text::truncate(&item.key_id, 80),
        source: text::truncate(&item.source, 40),
        country: text::truncate(&item.country, 40),
        title_text: item.title.clone(),
        label_text: text::truncate(&item.label, 40),
        revision: item.revision.to_string(),
        margin: format!("{:.2}", item.margin),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_redaction_notice_views(items: &[RedactionNotice]) -> Vec<RedactionNoticeView> {
    let mut views: Vec<RedactionNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_redaction_notice_view(item));
    }
    views
}

pub fn sum_redaction_notice_view(view: &RedactionNoticeView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_revision as f64 + parsed_margin)
}
