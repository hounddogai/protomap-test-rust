use super::*;

use acme_kit::text;

/// How screens and exports show a identity window.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityWindowView {
    pub code: String,
    pub label_text: String,
    pub active_text: String,
    pub margin_text: String,
    pub sequence: String,
    pub contact_email: String,
    pub amount_text: String,
    pub country: String,
    pub quantity: String,
    pub external_ref: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_identity_window_view(item: &IdentityWindow) -> IdentityWindowView {
    IdentityWindowView {
        code: text::truncate(&item.code, 120),
        label_text: item.label.clone(),
        active_text: format!("{}", item.active),
        margin_text: format!("{:.2}", item.margin),
        sequence: item.sequence.to_string(),
        contact_email: text::truncate(&item.contact_email, 120),
        amount_text: format!("{:.2}", item.amount),
        country: item.country.clone(),
        quantity: item.quantity.to_string(),
        external_ref: item.external_ref.clone(),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_window_views(items: &[IdentityWindow]) -> Vec<IdentityWindowView> {
    let mut views: Vec<IdentityWindowView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_window_view(item));
    }
    views
}

pub fn sum_identity_window_view(view: &IdentityWindowView) -> Result<f64, String> {
    let parsed_margin = view
        .margin_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_sequence = view
        .sequence
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_margin + (parsed_sequence as f64) + parsed_amount)
}
