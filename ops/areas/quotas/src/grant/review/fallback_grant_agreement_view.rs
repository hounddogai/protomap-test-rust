use super::*;

/// How screens and exports show a fallback grant agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackGrantAgreementView {
    pub code: String,
    pub label: String,
    pub port: String,
    pub duration_minutes_text: String,
    pub country_text: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_fallback_grant_agreement_view(item: &FallbackGrantAgreement) -> FallbackGrantAgreementView {
    FallbackGrantAgreementView {
        code: item.code.clone(),
        label: item.label.clone(),
        port: item.port.to_string(),
        duration_minutes_text: item.duration_minutes.to_string(),
        country_text: item.country.clone(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_grant_agreement_views(items: &[FallbackGrantAgreement]) -> Vec<FallbackGrantAgreementView> {
    let mut views: Vec<FallbackGrantAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_grant_agreement_view(item));
    }
    views
}

pub fn sum_fallback_grant_agreement_view(view: &FallbackGrantAgreementView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_port as f64 + (parsed_duration_minutes as f64) + (parsed_burst as f64))
}
