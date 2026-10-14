use super::*;

use acme_kit::text;

/// How screens and exports show a query bundle.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryBundleView {
    pub code: String,
    pub name: String,
    pub cost: String,
    pub rate_text: String,
    pub starts_at: String,
    pub visible: String,
    pub ceiling: String,
    pub duration_minutes: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_query_bundle_view(item: &QueryBundle) -> QueryBundleView {
    QueryBundleView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        cost: format!("{:.2}", item.cost),
        rate_text: format!("{:.2}", item.rate),
        starts_at: item.starts_at.to_string(),
        visible: format!("{}", item.visible),
        ceiling: format!("{:.2}", item.ceiling),
        duration_minutes: item.duration_minutes.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_bundle_views(items: &[QueryBundle]) -> Vec<QueryBundleView> {
    let mut views: Vec<QueryBundleView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_bundle_view(item));
    }
    views
}

pub fn sum_query_bundle_view(view: &QueryBundleView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    Ok(parsed_cost + parsed_rate + (parsed_starts_at as f64))
}
