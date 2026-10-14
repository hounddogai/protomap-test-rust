use super::*;

use acme_kit::text;

/// How screens and exports show a credential cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCycleView {
    pub code_text: String,
    pub name: String,
    pub floor_value: String,
    pub weight_text: String,
    pub unit: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_credential_cycle_view(item: &CredentialCycle) -> CredentialCycleView {
    CredentialCycleView {
        code_text: text::truncate(&item.code, 80),
        name: text::truncate(&item.name, 120),
        floor_value: format!("{:.2}", item.floor_value),
        weight_text: format!("{:.2}", item.weight),
        unit: item.unit.clone(),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_credential_cycle_views(items: &[CredentialCycle]) -> Vec<CredentialCycleView> {
    let mut views: Vec<CredentialCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_credential_cycle_view(item));
    }
    views
}

pub fn sum_credential_cycle_view(view: &CredentialCycleView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_weight = view
        .weight_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse weight: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_floor_value + parsed_weight + (parsed_retries as f64))
}
