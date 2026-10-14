use super::*;

use acme_kit::text;

/// How screens and exports show a isolation mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationMappingView {
    pub code: String,
    pub label: String,
    pub revision_text: String,
    pub quantity: String,
    pub key_id_text: String,
    pub status_label: String,
}

pub fn to_isolation_mapping_view(item: &IsolationMapping) -> IsolationMappingView {
    IsolationMappingView {
        code: item.code.clone(),
        label: item.label.clone(),
        revision_text: item.revision.to_string(),
        quantity: item.quantity.to_string(),
        key_id_text: text::truncate(&item.key_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_isolation_mapping_views(items: &[IsolationMapping]) -> Vec<IsolationMappingView> {
    let mut views: Vec<IsolationMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_isolation_mapping_view(item));
    }
    views
}

pub fn sum_isolation_mapping_view(view: &IsolationMappingView) -> Result<f64, String> {
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_quantity as f64))
}
