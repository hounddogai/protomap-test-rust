use super::*;

use acme_kit::text;

/// How screens and exports show a flow log variance.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogVarianceView {
    pub code: String,
    pub name_text: String,
    pub principal: String,
    pub audience_text: String,
    pub issuer: String,
    pub rate: String,
    pub cost: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_flow_log_variance_view(item: &FlowLogVariance) -> FlowLogVarianceView {
    FlowLogVarianceView {
        code: text::truncate(&item.code, 120),
        name_text: text::truncate(&item.name, 80),
        principal: item.principal.clone(),
        audience_text: text::truncate(&item.audience, 80),
        issuer: item.issuer.clone(),
        rate: format!("{:.2}", item.rate),
        cost: format!("{:.2}", item.cost),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_variance_views(items: &[FlowLogVariance]) -> Vec<FlowLogVarianceView> {
    let mut views: Vec<FlowLogVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_variance_view(item));
    }
    views
}

pub fn sum_flow_log_variance_view(view: &FlowLogVarianceView) -> Result<f64, String> {
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
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_rate + parsed_cost + (parsed_burst as f64))
}
