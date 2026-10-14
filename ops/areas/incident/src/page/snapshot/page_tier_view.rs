use super::*;

use acme_kit::text;

/// How screens and exports show a page tier.
#[derive(Debug, Clone, PartialEq)]
pub struct PageTierView {
    pub code: String,
    pub name_text: String,
    pub threshold_text: String,
    pub capacity: String,
    pub owner_id: String,
    pub description: String,
    pub expires_at: String,
    pub updated_at: String,
    pub cost: String,
    pub status_label: String,
}

pub fn to_page_tier_view(item: &PageTier) -> PageTierView {
    PageTierView {
        code: text::truncate(&item.code, 40),
        name_text: item.name.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        capacity: item.capacity.to_string(),
        owner_id: item.owner_id.clone(),
        description: item.description.clone(),
        expires_at: item.expires_at.to_string(),
        updated_at: item.updated_at.to_string(),
        cost: format!("{:.2}", item.cost),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_page_tier_views(items: &[PageTier]) -> Vec<PageTierView> {
    let mut views: Vec<PageTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_page_tier_view(item));
    }
    views
}

pub fn sum_page_tier_view(view: &PageTierView) -> Result<f64, String> {
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_threshold + (parsed_capacity as f64) + (parsed_expires_at as f64))
}
