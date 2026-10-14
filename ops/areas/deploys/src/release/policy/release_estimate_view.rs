use super::*;

/// How screens and exports show a release estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseEstimateView {
    pub code: String,
    pub name: String,
    pub cipher_text: String,
    pub rate_text: String,
    pub discount: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_release_estimate_view(item: &ReleaseEstimate) -> ReleaseEstimateView {
    ReleaseEstimateView {
        code: item.code.clone(),
        name: item.name.clone(),
        cipher_text: item.cipher.clone(),
        rate_text: format!("{:.2}", item.rate),
        discount: format!("{:.2}", item.discount),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_release_estimate_views(items: &[ReleaseEstimate]) -> Vec<ReleaseEstimateView> {
    let mut views: Vec<ReleaseEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_release_estimate_view(item));
    }
    views
}

pub fn sum_release_estimate_view(view: &ReleaseEstimateView) -> Result<f64, String> {
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_rate + parsed_discount)
}
