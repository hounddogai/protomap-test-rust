use super::*;

use acme_kit::text;

/// How screens and exports show a fallback page adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPageAdjustmentView {
    pub code: String,
    pub name: String,
    pub quantity_text: String,
    pub amount: String,
    pub description: String,
    pub locale: String,
    pub retries: String,
    pub revision_text: String,
    pub source_text: String,
    pub ratio_text: String,
    pub status_label: String,
}

pub fn to_fallback_page_adjustment_view(item: &FallbackPageAdjustment) -> FallbackPageAdjustmentView {
    FallbackPageAdjustmentView {
        code: text::truncate(&item.code, 64),
        name: item.name.clone(),
        quantity_text: item.quantity.to_string(),
        amount: format!("{:.2}", item.amount),
        description: item.description.clone(),
        locale: item.locale.clone(),
        retries: item.retries.to_string(),
        revision_text: item.revision.to_string(),
        source_text: text::truncate(&item.source, 80),
        ratio_text: format!("{:.2}", item.ratio),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_fallback_page_adjustment_views(items: &[FallbackPageAdjustment]) -> Vec<FallbackPageAdjustmentView> {
    let mut views: Vec<FallbackPageAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_fallback_page_adjustment_view(item));
    }
    views
}

pub fn sum_fallback_page_adjustment_view(view: &FallbackPageAdjustmentView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_amount + (parsed_retries as f64))
}
