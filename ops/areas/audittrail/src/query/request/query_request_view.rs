use super::*;

/// How screens and exports show a query request.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequestView {
    pub code_text: String,
    pub label: String,
    pub reviewer_text: String,
    pub weight: String,
    pub ceiling_text: String,
    pub capacity: String,
    pub batch_size: String,
    pub limit: String,
    pub status_label: String,
}

pub fn to_query_request_view(item: &QueryRequest) -> QueryRequestView {
    QueryRequestView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        reviewer_text: item.reviewer.clone(),
        weight: format!("{:.2}", item.weight),
        ceiling_text: format!("{:.2}", item.ceiling),
        capacity: item.capacity.to_string(),
        batch_size: item.batch_size.to_string(),
        limit: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_request_views(items: &[QueryRequest]) -> Vec<QueryRequestView> {
    let mut views: Vec<QueryRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_request_view(item));
    }
    views
}

pub fn sum_query_request_view(view: &QueryRequestView) -> Result<f64, String> {
    let parsed_weight = view
        .weight
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_weight + parsed_ceiling + (parsed_capacity as f64))
}
