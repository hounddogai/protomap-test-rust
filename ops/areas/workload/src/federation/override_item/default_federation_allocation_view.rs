use super::*;

use acme_kit::text;

/// How screens and exports show a default federation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFederationAllocationView {
    pub code: String,
    pub name: String,
    pub description_text: String,
    pub cost: String,
    pub segment_text: String,
    pub sequence_text: String,
    pub scope: String,
    pub active: String,
    pub maximum: String,
    pub revision: String,
    pub burst: String,
    pub status_label: String,
}

pub fn to_default_federation_allocation_view(item: &DefaultFederationAllocation) -> DefaultFederationAllocationView {
    DefaultFederationAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        description_text: text::truncate(&item.description, 40),
        cost: format!("{:.2}", item.cost),
        segment_text: item.segment.clone(),
        sequence_text: item.sequence.to_string(),
        scope: item.scope.clone(),
        active: format!("{}", item.active),
        maximum: format!("{:.2}", item.maximum),
        revision: item.revision.to_string(),
        burst: item.burst.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_federation_allocation_views(
    items: &[DefaultFederationAllocation],
) -> Vec<DefaultFederationAllocationView> {
    let mut views: Vec<DefaultFederationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_federation_allocation_view(item));
    }
    views
}

pub fn sum_default_federation_allocation_view(view: &DefaultFederationAllocationView) -> Result<f64, String> {
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_maximum = view
        .maximum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    Ok(parsed_cost + (parsed_sequence as f64) + parsed_maximum)
}
