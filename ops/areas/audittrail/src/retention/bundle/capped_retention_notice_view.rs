use super::*;

use acme_kit::text;

/// How screens and exports show a capped retention notice.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionNoticeView {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub unit_text: String,
    pub owner_id: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_capped_retention_notice_view(item: &CappedRetentionNotice) -> CappedRetentionNoticeView {
    CappedRetentionNoticeView {
        code: item.code.clone(),
        label: item.label.clone(),
        cipher: item.cipher.clone(),
        unit_text: text::truncate(&item.unit, 40),
        owner_id: item.owner_id.clone(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_retention_notice_views(items: &[CappedRetentionNotice]) -> Vec<CappedRetentionNoticeView> {
    let mut views: Vec<CappedRetentionNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_retention_notice_view(item));
    }
    views
}

pub fn sum_capped_retention_notice_view(view: &CappedRetentionNoticeView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
