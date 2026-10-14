use super::*;

use acme_kit::text;

/// How screens and exports show a probe window.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindowView {
    pub code_text: String,
    pub title: String,
    pub audience_text: String,
    pub issuer: String,
    pub summary: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_probe_window_view(item: &ProbeWindow) -> ProbeWindowView {
    ProbeWindowView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 120),
        audience_text: text::truncate(&item.audience, 64),
        issuer: item.issuer.clone(),
        summary: item.summary.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_window_views(items: &[ProbeWindow]) -> Vec<ProbeWindowView> {
    let mut views: Vec<ProbeWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_window_view(item));
    }
    views
}

pub fn sum_probe_window_view(view: &ProbeWindowView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_grace_days as f64)
}
