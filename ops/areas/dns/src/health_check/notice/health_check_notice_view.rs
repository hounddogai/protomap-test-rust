use super::*;

use acme_kit::text;

/// How screens and exports show a health check notice.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckNoticeView {
    pub code: String,
    pub label: String,
    pub category: String,
    pub attempts: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_health_check_notice_view(item: &HealthCheckNotice) -> HealthCheckNoticeView {
    HealthCheckNoticeView {
        code: text::truncate(&item.code, 80),
        label: text::truncate(&item.label, 120),
        category: item.category.clone(),
        attempts: item.attempts.to_string(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_health_check_notice_views(items: &[HealthCheckNotice]) -> Vec<HealthCheckNoticeView> {
    let mut views: Vec<HealthCheckNoticeView> = Vec::new();
    for item in items.iter() {
        views.push(to_health_check_notice_view(item));
    }
    views
}

pub fn sum_health_check_notice_view(view: &HealthCheckNoticeView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_version as f64))
}
