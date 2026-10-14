use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist digest.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistDigestView {
    pub code_text: String,
    pub title: String,
    pub burst: String,
    pub principal: String,
    pub audience: String,
    pub category_text: String,
    pub quantity_text: String,
    pub cipher_text: String,
    pub window_days: String,
    pub status_label: String,
}

pub fn to_allowlist_digest_view(item: &AllowlistDigest) -> AllowlistDigestView {
    AllowlistDigestView {
        code_text: text::truncate(&item.code, 40),
        title: item.title.clone(),
        burst: item.burst.to_string(),
        principal: item.principal.clone(),
        audience: item.audience.clone(),
        category_text: item.category.clone(),
        quantity_text: item.quantity.to_string(),
        cipher_text: item.cipher.clone(),
        window_days: item.window_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_digest_views(items: &[AllowlistDigest]) -> Vec<AllowlistDigestView> {
    let mut views: Vec<AllowlistDigestView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_digest_view(item));
    }
    views
}

pub fn sum_allowlist_digest_view(view: &AllowlistDigestView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    Ok(parsed_burst as f64 + (parsed_quantity as f64) + (parsed_window_days as f64))
}
