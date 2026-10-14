use super::*;

use acme_kit::text;

/// How screens and exports show a federation variance.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationVarianceView {
    pub code_text: String,
    pub name: String,
    pub discount: String,
    pub checksum_text: String,
    pub cluster: String,
    pub ceiling: String,
    pub audience: String,
    pub grace_days: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_federation_variance_view(item: &FederationVariance) -> FederationVarianceView {
    FederationVarianceView {
        code_text: text::truncate(&item.code, 64),
        name: item.name.clone(),
        discount: format!("{:.2}", item.discount),
        checksum_text: item.checksum.clone(),
        cluster: item.cluster.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        audience: text::truncate(&item.audience, 120),
        grace_days: item.grace_days.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_federation_variance_views(items: &[FederationVariance]) -> Vec<FederationVarianceView> {
    let mut views: Vec<FederationVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_federation_variance_view(item));
    }
    views
}

pub fn sum_federation_variance_view(view: &FederationVarianceView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_discount + parsed_ceiling + (parsed_grace_days as f64))
}
