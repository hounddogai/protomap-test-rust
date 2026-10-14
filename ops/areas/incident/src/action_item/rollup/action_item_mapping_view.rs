use super::*;

use acme_kit::text;

/// How screens and exports show a action item mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemMappingView {
    pub code_text: String,
    pub label: String,
    pub namespace: String,
    pub contact_email_text: String,
    pub currency: String,
    pub ceiling: String,
    pub attempts: String,
    pub threshold: String,
    pub status_label: String,
}

pub fn to_action_item_mapping_view(item: &ActionItemMapping) -> ActionItemMappingView {
    ActionItemMappingView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        namespace: item.namespace.clone(),
        contact_email_text: item.contact_email.clone(),
        currency: text::truncate(&item.currency, 80),
        ceiling: format!("{:.2}", item.ceiling),
        attempts: item.attempts.to_string(),
        threshold: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_action_item_mapping_views(items: &[ActionItemMapping]) -> Vec<ActionItemMappingView> {
    let mut views: Vec<ActionItemMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_action_item_mapping_view(item));
    }
    views
}

pub fn sum_action_item_mapping_view(view: &ActionItemMappingView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_ceiling + (parsed_attempts as f64) + parsed_threshold)
}
