use super::*;

/// How screens and exports show a quote profile.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteProfileView {
    pub code: String,
    pub name_text: String,
    pub cluster: String,
    pub version_text: String,
    pub country: String,
    pub burst: String,
    pub revision_text: String,
    pub attempts_text: String,
    pub status_label: String,
}

pub fn to_quote_profile_view(item: &QuoteProfile) -> QuoteProfileView {
    QuoteProfileView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        cluster: item.cluster.clone(),
        version_text: item.version.to_string(),
        country: item.country.clone(),
        burst: item.burst.to_string(),
        revision_text: item.revision.to_string(),
        attempts_text: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_quote_profile_views(items: &[QuoteProfile]) -> Vec<QuoteProfileView> {
    let mut views: Vec<QuoteProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_quote_profile_view(item));
    }
    views
}

pub fn sum_quote_profile_view(view: &QuoteProfileView) -> Result<f64, String> {
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_version as f64 + (parsed_burst as f64) + (parsed_revision as f64))
}
