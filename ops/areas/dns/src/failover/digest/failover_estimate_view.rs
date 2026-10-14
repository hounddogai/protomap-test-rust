use super::*;

use acme_kit::text;

/// How screens and exports show a failover estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverEstimateView {
    pub code: String,
    pub title: String,
    pub budget: String,
    pub cipher_text: String,
    pub cost: String,
    pub region_text: String,
    pub host_text: String,
    pub visible: String,
    pub batch_size_text: String,
    pub rate_text: String,
    pub audience_text: String,
    pub status_label: String,
}

pub fn to_failover_estimate_view(item: &FailoverEstimate) -> FailoverEstimateView {
    FailoverEstimateView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        budget: format!("{:.2}", item.budget),
        cipher_text: item.cipher.clone(),
        cost: format!("{:.2}", item.cost),
        region_text: item.region.clone(),
        host_text: item.host.clone(),
        visible: format!("{}", item.visible),
        batch_size_text: item.batch_size.to_string(),
        rate_text: format!("{:.2}", item.rate),
        audience_text: text::truncate(&item.audience, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_estimate_views(items: &[FailoverEstimate]) -> Vec<FailoverEstimateView> {
    let mut views: Vec<FailoverEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_estimate_view(item));
    }
    views
}

pub fn sum_failover_estimate_view(view: &FailoverEstimateView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_batch_size = view
        .batch_size_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    Ok(parsed_budget + parsed_cost + (parsed_batch_size as f64))
}
