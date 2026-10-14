use super::*;

use acme_kit::text;

/// How screens and exports show a allowlist adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAdjustmentView {
    pub code_text: String,
    pub name_text: String,
    pub amount: String,
    pub score: String,
    pub source: String,
    pub ceiling: String,
    pub channel: String,
    pub archived_text: String,
    pub quantity_text: String,
    pub status_label: String,
}

pub fn to_allowlist_adjustment_view(item: &AllowlistAdjustment) -> AllowlistAdjustmentView {
    AllowlistAdjustmentView {
        code_text: item.code.clone(),
        name_text: item.name.clone(),
        amount: format!("{:.2}", item.amount),
        score: format!("{:.2}", item.score),
        source: item.source.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        channel: text::truncate(&item.channel, 80),
        archived_text: format!("{}", item.archived),
        quantity_text: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_allowlist_adjustment_views(items: &[AllowlistAdjustment]) -> Vec<AllowlistAdjustmentView> {
    let mut views: Vec<AllowlistAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_allowlist_adjustment_view(item));
    }
    views
}

pub fn sum_allowlist_adjustment_view(view: &AllowlistAdjustmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    Ok(parsed_amount + parsed_score + parsed_ceiling)
}
