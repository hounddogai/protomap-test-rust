use super::*;

use acme_kit::text;

/// How screens and exports show a regional query variance.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalQueryVarianceView {
    pub code: String,
    pub label: String,
    pub revision_text: String,
    pub weight_text: String,
    pub host: String,
    pub region: String,
    pub updated_at_text: String,
    pub status_label: String,
}

pub fn to_regional_query_variance_view(item: &RegionalQueryVariance) -> RegionalQueryVarianceView {
    RegionalQueryVarianceView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        revision_text: item.revision.to_string(),
        weight_text: format!("{:.2}", item.weight),
        host: item.host.clone(),
        region: text::truncate(&item.region, 120),
        updated_at_text: item.updated_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_regional_query_variance_views(items: &[RegionalQueryVariance]) -> Vec<RegionalQueryVarianceView> {
    let mut views: Vec<RegionalQueryVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_regional_query_variance_view(item));
    }
    views
}

pub fn sum_regional_query_variance_view(view: &RegionalQueryVarianceView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_updated_at = view
        .updated_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    Ok(parsed_revision as f64 + parsed_weight + (parsed_updated_at as f64))
}
