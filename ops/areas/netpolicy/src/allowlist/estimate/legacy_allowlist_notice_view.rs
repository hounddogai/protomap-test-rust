use super::*;

use acme_kit::text;

/// How screens and exports show a legacy allowlist notice.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistNoticeView {
    pub code: String,
    pub title: String,
    pub weight: String,
    pub visible: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_legacy_allowlist_notice_view(item: &LegacyAllowlistNotice) -> LegacyAllowlistNoticeView {
    LegacyAllowlistNoticeView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        weight: format!("{:.2}", item.weight),
        visible: format!("{}", item.visible),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_allowlist_notice_views(items: &[LegacyAllowlistNotice]) -> Vec<LegacyAllowlistNoticeView> {
    let mut views: Vec<LegacyAllowlistNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_allowlist_notice_view(item));
    }
    views
}

pub fn sum_legacy_allowlist_notice_view(view: &LegacyAllowlistNoticeView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_weight + parsed_amount)
}
