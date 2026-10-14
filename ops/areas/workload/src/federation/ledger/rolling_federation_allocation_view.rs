use super::*;

/// How screens and exports show a rolling federation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationAllocationView {
    pub code_text: String,
    pub label: String,
    pub margin: String,
    pub quantity: String,
    pub port: String,
    pub fingerprint: String,
    pub floor_value_text: String,
    pub status_label: String,
}

pub fn to_rolling_federation_allocation_view(item: &RollingFederationAllocation) -> RollingFederationAllocationView {
    RollingFederationAllocationView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        margin: format!("{:.2}", item.margin),
        quantity: item.quantity.to_string(),
        port: item.port.to_string(),
        fingerprint: item.fingerprint.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_federation_allocation_views(
    items: &[RollingFederationAllocation],
) -> Vec<RollingFederationAllocationView> {
    let mut views: Vec<RollingFederationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_federation_allocation_view(item));
    }
    views
}

pub fn sum_rolling_federation_allocation_view(view: &RollingFederationAllocationView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_margin + (parsed_quantity as f64) + (parsed_port as f64))
}
