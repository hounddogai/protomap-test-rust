use super::*;

/// How screens and exports show a fallback release exception.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseExceptionView {
    pub code: String,
    pub name: String,
    pub quota_units: String,
    pub ttl_seconds_text: String,
    pub batch_size_text: String,
    pub issuer: String,
    pub grace_days_text: String,
    pub maximum_text: String,
    pub status_label: String,
}

pub fn to_fallback_release_exception_view(item: &FallbackReleaseException) -> FallbackReleaseExceptionView {
    FallbackReleaseExceptionView {
        code: item.code.clone(),
        name: item.name.clone(),
        quota_units: item.quota_units.to_string(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        batch_size_text: item.batch_size.to_string(),
        issuer: item.issuer.clone(),
        grace_days_text: item.grace_days.to_string(),
        maximum_text: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_release_exception_views(items: &[FallbackReleaseException]) -> Vec<FallbackReleaseExceptionView> {
    let mut views: Vec<FallbackReleaseExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_release_exception_view(item));
    }
    views
}

pub fn sum_fallback_release_exception_view(view: &FallbackReleaseExceptionView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_quota_units as f64 + (parsed_ttl_seconds as f64) + (parsed_batch_size as f64))
}
