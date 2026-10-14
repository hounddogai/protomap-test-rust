use super::*;

use acme_kit::text;

/// How screens and exports show a suspension summary.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSummaryView {
    pub code: String,
    pub title_text: String,
    pub expires_at_text: String,
    pub rotation_days: String,
    pub grace_days: String,
    pub window_days_text: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_suspension_summary_view(item: &SuspensionSummary) -> SuspensionSummaryView {
    SuspensionSummaryView {
        code: item.code.clone(),
        title_text: text::truncate(&item.title, 40),
        expires_at_text: item.expires_at.to_string(),
        rotation_days: item.rotation_days.to_string(),
        grace_days: item.grace_days.to_string(),
        window_days_text: item.window_days.to_string(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_summary_views(items: &[SuspensionSummary]) -> Vec<SuspensionSummaryView> {
    let mut views: Vec<SuspensionSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_summary_view(item));
    }
    views
}

pub fn sum_suspension_summary_view(view: &SuspensionSummaryView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_rotation_days as f64) + (parsed_grace_days as f64))
}
