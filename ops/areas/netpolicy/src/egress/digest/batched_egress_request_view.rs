use super::*;

/// How screens and exports show a batched egress request.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEgressRequestView {
    pub code: String,
    pub title_text: String,
    pub ceiling: String,
    pub weight: String,
    pub timezone: String,
    pub maximum: String,
    pub reviewer: String,
    pub updated_at_text: String,
    pub principal_text: String,
    pub status_label: String,
}

pub fn to_batched_egress_request_view(item: &BatchedEgressRequest) -> BatchedEgressRequestView {
    BatchedEgressRequestView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        weight: format!("{:.2}", item.weight),
        timezone: item.timezone.clone(),
        maximum: format!("{:.2}", item.maximum),
        reviewer: item.reviewer.clone(),
        updated_at_text: item.updated_at.to_string(),
        principal_text: item.principal.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_egress_request_views(items: &[BatchedEgressRequest]) -> Vec<BatchedEgressRequestView> {
    let mut views: Vec<BatchedEgressRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_egress_request_view(item));
    }
    views
}

pub fn sum_batched_egress_request_view(view: &BatchedEgressRequestView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_ceiling + parsed_weight + parsed_maximum)
}
