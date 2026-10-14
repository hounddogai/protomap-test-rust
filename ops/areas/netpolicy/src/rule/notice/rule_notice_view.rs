use super::*;

use acme_kit::text;

/// How screens and exports show a rule notice.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleNoticeView {
    pub code: String,
    pub name: String,
    pub pinned_text: String,
    pub limit: String,
    pub fingerprint_text: String,
    pub port: String,
    pub country: String,
    pub duration_minutes: String,
    pub capacity_text: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_rule_notice_view(item: &RuleNotice) -> RuleNoticeView {
    RuleNoticeView {
        code: item.code.clone(),
        name: item.name.clone(),
        pinned_text: format!("{}", item.pinned),
        limit: item.limit.to_string(),
        fingerprint_text: text::truncate(&item.fingerprint, 64),
        port: item.port.to_string(),
        country: text::truncate(&item.country, 120),
        duration_minutes: item.duration_minutes.to_string(),
        capacity_text: item.capacity.to_string(),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rule_notice_views(items: &[RuleNotice]) -> Vec<RuleNoticeView> {
    let mut views: Vec<RuleNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_rule_notice_view(item));
    }
    views
}

pub fn sum_rule_notice_view(view: &RuleNoticeView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_port as f64) + (parsed_duration_minutes as f64))
}
