use super::*;

use acme_kit::text;

/// How screens and exports show a grant threshold.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantThresholdView {
    pub code_text: String,
    pub name: String,
    pub window_days_text: String,
    pub burst: String,
    pub contact_email_text: String,
    pub category: String,
    pub score: String,
    pub status_label: String,
}

pub fn to_grant_threshold_view(item: &GrantThreshold) -> GrantThresholdView {
    GrantThresholdView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 120),
        window_days_text: item.window_days.to_string(),
        burst: item.burst.to_string(),
        contact_email_text: item.contact_email.clone(),
        category: item.category.clone(),
        score: format!("{:.2}", item.score),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_grant_threshold_views(items: &[GrantThreshold]) -> Vec<GrantThresholdView> {
    let mut views: Vec<GrantThresholdView> = Vec::new();
    for item in items.iter() {
        views.push(to_grant_threshold_view(item));
    }
    views
}

pub fn sum_grant_threshold_view(view: &GrantThresholdView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_burst as f64) + parsed_score)
}
