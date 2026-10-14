use super::*;

use acme_kit::text;

/// How screens and exports show a global token override.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTokenOverrideView {
    pub code: String,
    pub label: String,
    pub cluster_text: String,
    pub burst: String,
    pub country: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_global_token_override_view(item: &GlobalTokenOverride) -> GlobalTokenOverrideView {
    GlobalTokenOverrideView {
        code: text::truncate(&item.code, 120),
        label: text::truncate(&item.label, 64),
        cluster_text: text::truncate(&item.cluster, 80),
        burst: item.burst.to_string(),
        country: item.country.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_token_override_views(items: &[GlobalTokenOverride]) -> Vec<GlobalTokenOverrideView> {
    let mut views: Vec<GlobalTokenOverrideView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_token_override_view(item));
    }
    views
}

pub fn sum_global_token_override_view(view: &GlobalTokenOverrideView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_quantity as f64))
}
