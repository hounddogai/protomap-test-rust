use super::*;

use acme_kit::text;

/// How screens and exports show a nonce variance.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceVarianceView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub key_id_text: String,
    pub notes: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_nonce_variance_view(item: &NonceVariance) -> NonceVarianceView {
    NonceVarianceView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        cipher: item.cipher.clone(),
        key_id_text: item.key_id.clone(),
        notes: text::truncate(&item.notes, 40),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_nonce_variance_views(items: &[NonceVariance]) -> Vec<NonceVarianceView> {
    let mut views: Vec<NonceVarianceView> = Vec::new();
    for item in items.iter() {
        views.push(to_nonce_variance_view(item));
    }
    views
}

pub fn sum_nonce_variance_view(view: &NonceVarianceView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
