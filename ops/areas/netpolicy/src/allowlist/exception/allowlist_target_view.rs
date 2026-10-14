use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist target.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetView {
    pub code: String,
    pub label: String,
    pub amount_text: String,
    pub currency_text: String,
    pub port_text: String,
    pub minimum: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_allowlist_target_view(item: &AllowlistTarget) -> AllowlistTargetView {
    AllowlistTargetView {
        code: text::truncate(&item.code, 64),
        label: text::truncate(&item.label, 120),
        amount_text: format!("{:.2}", item.amount),
        currency_text: item.currency.clone(),
        port_text: item.port.to_string(),
        minimum: format!("{:.2}", item.minimum),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_target_views(items: &[AllowlistTarget]) -> Vec<AllowlistTargetView> {
    let mut views: Vec<AllowlistTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_target_view(item));
    }
    views
}

pub fn sum_allowlist_target_view(view: &AllowlistTargetView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_port = view
        .port_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_amount + (parsed_port as f64) + parsed_minimum)
}
