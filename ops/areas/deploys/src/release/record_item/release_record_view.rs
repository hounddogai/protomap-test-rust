use super::*;

use acme_kit::text;

/// How screens and exports show a release record.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseRecordView {
    pub code: String,
    pub name: String,
    pub ttl_seconds_text: String,
    pub principal: String,
    pub phone: String,
    pub approved: String,
    pub status_label: String,
}

pub fn to_release_record_view(item: &ReleaseRecord) -> ReleaseRecordView {
    ReleaseRecordView {
        code: item.code.clone(),
        name: item.name.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        principal: text::truncate(&item.principal, 40),
        phone: item.phone.clone(),
        approved: format!("{}", item.approved),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_record_views(items: &[ReleaseRecord]) -> Vec<ReleaseRecordView> {
    let mut views: Vec<ReleaseRecordView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_record_view(item));
    }
    views
}

pub fn sum_release_record_view(view: &ReleaseRecordView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ttl_seconds as f64)
}
