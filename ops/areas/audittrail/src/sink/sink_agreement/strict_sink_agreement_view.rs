use super::*;

use acme_kit::text;

/// How screens and exports show a strict sink agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSinkAgreementView {
    pub code: String,
    pub title: String,
    pub window_days: String,
    pub tenant_id_text: String,
    pub active: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_strict_sink_agreement_view(item: &StrictSinkAgreement) -> StrictSinkAgreementView {
    StrictSinkAgreementView {
        code: text::truncate(&item.code, 80),
        title: item.title.clone(),
        window_days: item.window_days.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        active: format!("{}", item.active),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_sink_agreement_views(items: &[StrictSinkAgreement]) -> Vec<StrictSinkAgreementView> {
    let mut views: Vec<StrictSinkAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_sink_agreement_view(item));
    }
    views
}

pub fn sum_strict_sink_agreement_view(view: &StrictSinkAgreementView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_window_days as f64 + (parsed_quota_units as f64))
}
