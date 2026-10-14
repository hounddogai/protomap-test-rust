use super::*;

/// How screens and exports show a flow log estimate.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogEstimateView {
    pub code: String,
    pub label: String,
    pub version: String,
    pub score: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_flow_log_estimate_view(item: &FlowLogEstimate) -> FlowLogEstimateView {
    FlowLogEstimateView {
        code: item.code.clone(),
        label: item.label.clone(),
        version: item.version.to_string(),
        score: format!("{:.2}", item.score),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_flow_log_estimate_views(items: &[FlowLogEstimate]) -> Vec<FlowLogEstimateView> {
    let mut views: Vec<FlowLogEstimateView> = Vec::new();
    for item in items.iter() {
        views.push(to_flow_log_estimate_view(item));
    }
    views
}

pub fn sum_flow_log_estimate_view(view: &FlowLogEstimateView) -> Result<f64, String> {
    let parsed_version = view
        .version
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_version as f64 + parsed_score + parsed_amount)
}
