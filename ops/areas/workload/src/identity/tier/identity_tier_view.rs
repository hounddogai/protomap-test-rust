use super::*;

use acme_kit::text;

/// How screens and exports show a identity tier.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityTierView {
    pub code: String,
    pub title: String,
    pub name: String,
    pub host: String,
    pub active_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_identity_tier_view(item: &IdentityTier) -> IdentityTierView {
    IdentityTierView {
        code: item.code.clone(),
        title: text::truncate(&item.title, 64),
        name: item.name.clone(),
        host: text::truncate(&item.host, 80),
        active_text: format!("{}", item.active),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_identity_tier_views(items: &[IdentityTier]) -> Vec<IdentityTierView> {
    let mut views: Vec<IdentityTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_identity_tier_view(item));
    }
    views
}

pub fn sum_identity_tier_view(view: &IdentityTierView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
