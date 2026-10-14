use super::*;

/// How screens and exports show a pending retention segment.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionSegmentView {
    pub code: String,
    pub name_text: String,
    pub host: String,
    pub contact_email: String,
    pub approved: String,
    pub cipher_text: String,
    pub margin_text: String,
    pub status_label: String,
}

pub fn to_pending_retention_segment_view(item: &PendingRetentionSegment) -> PendingRetentionSegmentView {
    PendingRetentionSegmentView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        host: item.host.clone(),
        contact_email: item.contact_email.clone(),
        approved: format!("{}", item.approved),
        cipher_text: item.cipher.clone(),
        margin_text: format!("{:.2}", item.margin),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_retention_segment_views(items: &[PendingRetentionSegment]) -> Vec<PendingRetentionSegmentView> {
    let mut views: Vec<PendingRetentionSegmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_retention_segment_view(item));
    }
    views
}

pub fn sum_pending_retention_segment_view(view: &PendingRetentionSegmentView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_margin)
}
