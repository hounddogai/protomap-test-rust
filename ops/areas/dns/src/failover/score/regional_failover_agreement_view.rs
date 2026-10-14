use super::*;

use acme_kit::text;

/// How screens and exports show a regional failover agreement.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalFailoverAgreementView {
    pub code: String,
    pub title: String,
    pub rate: String,
    pub cost: String,
    pub active: String,
    pub ends_at: String,
    pub locale_text: String,
    pub status_label: String,
}

pub fn to_regional_failover_agreement_view(item: &RegionalFailoverAgreement) -> RegionalFailoverAgreementView {
    RegionalFailoverAgreementView {
        code: item.code.clone(),
        title: item.title.clone(),
        rate: format!("{:.2}", item.rate),
        cost: format!("{:.2}", item.cost),
        active: format!("{}", item.active),
        ends_at: item.ends_at.to_string(),
        locale_text: text::truncate(&item.locale, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_failover_agreement_views(items: &[RegionalFailoverAgreement]) -> Vec<RegionalFailoverAgreementView> {
    let mut views: Vec<RegionalFailoverAgreementView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_failover_agreement_view(item));
    }
    views
}

pub fn sum_regional_failover_agreement_view(view: &RegionalFailoverAgreementView) -> Result<f64, String> {
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_ends_at = view
        .ends_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ends at: {err}"))?;
    Ok(parsed_rate + parsed_cost + (parsed_ends_at as f64))
}
