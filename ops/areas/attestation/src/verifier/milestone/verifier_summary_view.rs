use super::*;

/// How screens and exports show a verifier summary.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierSummaryView {
    pub code: String,
    pub title: String,
    pub priority_text: String,
    pub name_text: String,
    pub version: String,
    pub status_label: String,
}

pub fn to_verifier_summary_view(item: &VerifierSummary) -> VerifierSummaryView {
    VerifierSummaryView {
        code: item.code.clone(),
        title: item.title.clone(),
        priority_text: item.priority.to_string(),
        name_text: item.name.clone(),
        version: item.version.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_summary_views(items: &[VerifierSummary]) -> Vec<VerifierSummaryView> {
    let mut views: Vec<VerifierSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_summary_view(item));
    }
    views
}

pub fn sum_verifier_summary_view(view: &VerifierSummaryView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_version as f64))
}
