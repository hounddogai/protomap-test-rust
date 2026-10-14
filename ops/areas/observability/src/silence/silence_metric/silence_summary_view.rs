use super::*;

use acme_kit::text;

/// How screens and exports show a silence summary.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSummaryView {
    pub code_text: String,
    pub label_text: String,
    pub cipher: String,
    pub starts_at: String,
    pub budget: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_silence_summary_view(item: &SilenceSummary) -> SilenceSummaryView {
    SilenceSummaryView {
        code_text: text::truncate(&item.code, 40),
        label_text: item.label.clone(),
        cipher: text::truncate(&item.cipher, 120),
        starts_at: item.starts_at.to_string(),
        budget: format!("{:.2}", item.budget),
        contact_email_text: text::truncate(&item.contact_email, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_summary_views(items: &[SilenceSummary]) -> Vec<SilenceSummaryView> {
    let mut views: Vec<SilenceSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_summary_view(item));
    }
    views
}

pub fn sum_silence_summary_view(view: &SilenceSummaryView) -> Result<f64, String> {
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_starts_at as f64 + parsed_budget)
}
