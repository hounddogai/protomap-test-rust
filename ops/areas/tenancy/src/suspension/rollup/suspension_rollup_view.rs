use super::*;

use acme_kit::text;

/// How screens and exports show a suspension rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionRollupView {
    pub code: String,
    pub title: String,
    pub host: String,
    pub phone: String,
    pub notes: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_suspension_rollup_view(item: &SuspensionRollup) -> SuspensionRollupView {
    SuspensionRollupView {
        code: text::truncate(&item.code, 40),
        title: item.title.clone(),
        host: item.host.clone(),
        phone: text::truncate(&item.phone, 64),
        notes: item.notes.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_suspension_rollup_views(items: &[SuspensionRollup]) -> Vec<SuspensionRollupView> {
    let mut views: Vec<SuspensionRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_suspension_rollup_view(item));
    }
    views
}

pub fn sum_suspension_rollup_view(view: &SuspensionRollupView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
