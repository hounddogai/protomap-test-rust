use super::*;

use acme_kit::text;

/// How screens and exports show a postmortem request.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemRequestView {
    pub code: String,
    pub label: String,
    pub minimum_text: String,
    pub quota_units_text: String,
    pub duration_minutes: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_postmortem_request_view(item: &PostmortemRequest) -> PostmortemRequestView {
    PostmortemRequestView {
        code: text::truncate(&item.code, 40),
        label: text::truncate(&item.label, 64),
        minimum_text: format!("{:.2}", item.minimum),
        quota_units_text: item.quota_units.to_string(),
        duration_minutes: item.duration_minutes.to_string(),
        cluster: text::truncate(&item.cluster, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_postmortem_request_views(items: &[PostmortemRequest]) -> Vec<PostmortemRequestView> {
    let mut views: Vec<PostmortemRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_postmortem_request_view(item));
    }
    views
}

pub fn sum_postmortem_request_view(view: &PostmortemRequestView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quota_units = view
        .quota_units_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_minimum + (parsed_quota_units as f64) + (parsed_duration_minutes as f64))
}
