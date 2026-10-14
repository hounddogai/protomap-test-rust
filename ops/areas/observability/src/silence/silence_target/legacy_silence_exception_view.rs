use super::*;

use acme_kit::text;

/// How screens and exports show a legacy silence exception.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySilenceExceptionView {
    pub code: String,
    pub name_text: String,
    pub grace_days_text: String,
    pub priority_text: String,
    pub margin: String,
    pub checksum: String,
    pub status_label: String,
}

pub fn to_legacy_silence_exception_view(item: &LegacySilenceException) -> LegacySilenceExceptionView {
    LegacySilenceExceptionView {
        code: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 64),
        grace_days_text: item.grace_days.to_string(),
        priority_text: item.priority.to_string(),
        margin: format!("{:.2}", item.margin),
        checksum: item.checksum.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_silence_exception_views(items: &[LegacySilenceException]) -> Vec<LegacySilenceExceptionView> {
    let mut views: Vec<LegacySilenceExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_silence_exception_view(item));
    }
    views
}

pub fn sum_legacy_silence_exception_view(view: &LegacySilenceExceptionView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_priority as f64) + parsed_margin)
}
