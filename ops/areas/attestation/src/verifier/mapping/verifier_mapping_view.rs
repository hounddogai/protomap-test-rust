use super::*;

/// How screens and exports show a verifier mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMappingView {
    pub code: String,
    pub label: String,
    pub rate: String,
    pub retries: String,
    pub grace_days: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_verifier_mapping_view(item: &VerifierMapping) -> VerifierMappingView {
    VerifierMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        rate: format!("{:.2}", item.rate),
        retries: item.retries.to_string(),
        grace_days: item.grace_days.to_string(),
        locale_text: item.locale.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_verifier_mapping_views(items: &[VerifierMapping]) -> Vec<VerifierMappingView> {
    let mut views: Vec<VerifierMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_verifier_mapping_view(item));
    }
    views
}

pub fn sum_verifier_mapping_view(view: &VerifierMappingView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_rate + (parsed_retries as f64) + (parsed_grace_days as f64))
}
