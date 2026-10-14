use super::*;

/// How screens and exports show a nonce request.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceRequestView {
    pub code: String,
    pub label_text: String,
    pub title_text: String,
    pub minimum: String,
    pub quantity: String,
    pub budget: String,
    pub status_label: String,
}

pub fn to_nonce_request_view(item: &NonceRequest) -> NonceRequestView {
    NonceRequestView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        title_text: item.title.clone(),
        minimum: format!("{:.2}", item.minimum),
        quantity: item.quantity.to_string(),
        budget: format!("{:.2}", item.budget),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_request_views(items: &[NonceRequest]) -> Vec<NonceRequestView> {
    let mut views: Vec<NonceRequestView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_request_view(item));
    }
    views
}

pub fn sum_nonce_request_view(view: &NonceRequestView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_minimum + (parsed_quantity as f64) + parsed_budget)
}
