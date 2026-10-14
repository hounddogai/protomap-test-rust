use super::*;

use acme_kit::text;

/// How screens and exports show a weighted grant notice.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantNoticeView {
    pub code_text: String,
    pub label: String,
    pub created_at: String,
    pub minimum_text: String,
    pub retries_text: String,
    pub pinned: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_weighted_grant_notice_view(item: &WeightedGrantNotice) -> WeightedGrantNoticeView {
    WeightedGrantNoticeView {
        code_text: text::truncate(&item.code, 64),
        label: item.label.clone(),
        created_at: item.created_at.to_string(),
        minimum_text: format!("{:.2}", item.minimum),
        retries_text: item.retries.to_string(),
        pinned: format!("{}", item.pinned),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_grant_notice_views(items: &[WeightedGrantNotice]) -> Vec<WeightedGrantNoticeView> {
    let mut views: Vec<WeightedGrantNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_grant_notice_view(item));
    }
    views
}

pub fn sum_weighted_grant_notice_view(view: &WeightedGrantNoticeView) -> Result<f64, String> {
    let parsed_created_at = view
        .created_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse created at: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_retries = view
        .retries_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_created_at as f64 + parsed_minimum + (parsed_retries as f64))
}
