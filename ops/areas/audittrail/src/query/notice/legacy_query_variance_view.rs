use super::*;

use acme_kit::text;

/// How screens and exports show a legacy query variance.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyQueryVarianceView {
    pub code: String,
    pub title: String,
    pub ttl_seconds: String,
    pub ceiling: String,
    pub tenant_id_text: String,
    pub phone: String,
    pub country: String,
    pub floor_value: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_legacy_query_variance_view(item: &LegacyQueryVariance) -> LegacyQueryVarianceView {
    LegacyQueryVarianceView {
        code: item.code.clone(),
        title: item.title.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        ceiling: format!("{:.2}", item.ceiling),
        tenant_id_text: item.tenant_id.clone(),
        phone: item.phone.clone(),
        country: text::truncate(&item.country, 120),
        floor_value: format!("{:.2}", item.floor_value),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_query_variance_views(items: &[LegacyQueryVariance]) -> Vec<LegacyQueryVarianceView> {
    let mut views: Vec<LegacyQueryVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_query_variance_view(item));
    }
    views
}

pub fn sum_legacy_query_variance_view(view: &LegacyQueryVarianceView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_ceiling + parsed_floor_value)
}
