use super::*;

use acme_kit::text;

/// How screens and exports show a rolling page entry.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPageEntryView {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub active_text: String,
    pub audience: String,
    pub category_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_rolling_page_entry_view(item: &RollingPageEntry) -> RollingPageEntryView {
    RollingPageEntryView {
        code: item.code.clone(),
        title: item.title.clone(),
        cipher: text::truncate(&item.cipher, 40),
        active_text: format!("{}", item.active),
        audience: item.audience.clone(),
        category_text: item.category.clone(),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_page_entry_views(items: &[RollingPageEntry]) -> Vec<RollingPageEntryView> {
    let mut views: Vec<RollingPageEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_page_entry_view(item));
    }
    views
}

pub fn sum_rolling_page_entry_view(view: &RollingPageEntryView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
