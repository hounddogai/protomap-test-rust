use super::*;

use acme_kit::text;

/// How screens and exports show a regional verifier summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalVerifierSummaryView {
    pub code_text: String,
    pub name: String,
    pub phone: String,
    pub score_text: String,
    pub cipher: String,
    pub priority: String,
    pub title: String,
    pub status_label: String,
}

pub fn to_regional_verifier_summary_view(item: &RegionalVerifierSummary) -> RegionalVerifierSummaryView {
    RegionalVerifierSummaryView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 40),
        phone: item.phone.clone(),
        score_text: format!("{:.2}", item.score),
        cipher: item.cipher.clone(),
        priority: item.priority.to_string(),
        title: item.title.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_verifier_summary_views(items: &[RegionalVerifierSummary]) -> Vec<RegionalVerifierSummaryView> {
    let mut views: Vec<RegionalVerifierSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_verifier_summary_view(item));
    }
    views
}

pub fn sum_regional_verifier_summary_view(view: &RegionalVerifierSummaryView) -> Result<f64, String> {
    let parsed_score = view
        .score_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_score + (parsed_priority as f64))
}
