use super::*;

use acme_kit::text;

/// How screens and exports show a strict federation profile.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationProfileView {
    pub code: String,
    pub title: String,
    pub amount_text: String,
    pub grace_days: String,
    pub host_text: String,
    pub visible: String,
    pub limit: String,
    pub category_text: String,
    pub status_label: String,
}

pub fn to_strict_federation_profile_view(item: &StrictFederationProfile) -> StrictFederationProfileView {
    StrictFederationProfileView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount_text: format!("{:.2}", item.amount),
        grace_days: item.grace_days.to_string(),
        host_text: item.host.clone(),
        visible: format!("{}", item.visible),
        limit: item.limit.to_string(),
        category_text: text::truncate(&item.category, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_federation_profile_views(items: &[StrictFederationProfile]) -> Vec<StrictFederationProfileView> {
    let mut views: Vec<StrictFederationProfileView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_federation_profile_view(item));
    }
    views
}

pub fn sum_strict_federation_profile_view(view: &StrictFederationProfileView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_amount + (parsed_grace_days as f64) + (parsed_limit as f64))
}
