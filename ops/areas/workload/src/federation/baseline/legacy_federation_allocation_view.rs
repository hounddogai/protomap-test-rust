use super::*;

use acme_kit::text;

/// How screens and exports show a legacy federation allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationAllocationView {
    pub code: String,
    pub name: String,
    pub owner_id_text: String,
    pub discount: String,
    pub cipher: String,
    pub visible: String,
    pub status_label: String,
}

pub fn to_legacy_federation_allocation_view(item: &LegacyFederationAllocation) -> LegacyFederationAllocationView {
    LegacyFederationAllocationView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 120),
        owner_id_text: text::truncate(&item.owner_id, 40),
        discount: format!("{:.2}", item.discount),
        cipher: item.cipher.clone(),
        visible: format!("{}", item.visible),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_legacy_federation_allocation_views(
    items: &[LegacyFederationAllocation],
) -> Vec<LegacyFederationAllocationView> {
    let mut views: Vec<LegacyFederationAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_legacy_federation_allocation_view(item));
    }
    views
}

pub fn sum_legacy_federation_allocation_view(view: &LegacyFederationAllocationView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_discount)
}
