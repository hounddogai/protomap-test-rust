use super::*;

/// How screens and exports show a weighted federation bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationBundleView {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub updated_at: String,
    pub description: String,
    pub category_text: String,
    pub phone: String,
    pub rate: String,
    pub ratio: String,
    pub owner_id: String,
    pub attempts: String,
    pub status_label: String,
}

pub fn to_weighted_federation_bundle_view(item: &WeightedFederationBundle) -> WeightedFederationBundleView {
    WeightedFederationBundleView {
        code: item.code.clone(),
        name: item.name.clone(),
        summary: item.summary.clone(),
        updated_at: item.updated_at.to_string(),
        description: item.description.clone(),
        category_text: item.category.clone(),
        phone: item.phone.clone(),
        rate: format!("{:.2}", item.rate),
        ratio: format!("{:.2}", item.ratio),
        owner_id: item.owner_id.clone(),
        attempts: item.attempts.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_weighted_federation_bundle_views(items: &[WeightedFederationBundle]) -> Vec<WeightedFederationBundleView> {
    let mut views: Vec<WeightedFederationBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_weighted_federation_bundle_view(item));
    }
    views
}

pub fn sum_weighted_federation_bundle_view(view: &WeightedFederationBundleView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_rate + parsed_ratio)
}
