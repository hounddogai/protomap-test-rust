use super::*;

use acme_kit::text;

/// How screens and exports show a default sink adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSinkAdjustmentView {
    pub code: String,
    pub title_text: String,
    pub unit: String,
    pub owner_id: String,
    pub segment: String,
    pub amount_text: String,
    pub status_label: String,
}

pub fn to_default_sink_adjustment_view(item: &DefaultSinkAdjustment) -> DefaultSinkAdjustmentView {
    DefaultSinkAdjustmentView {
        code: text::truncate(&item.code, 64),
        title_text: item.title.clone(),
        unit: item.unit.clone(),
        owner_id: item.owner_id.clone(),
        segment: text::truncate(&item.segment, 64),
        amount_text: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_sink_adjustment_views(items: &[DefaultSinkAdjustment]) -> Vec<DefaultSinkAdjustmentView> {
    let mut views: Vec<DefaultSinkAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_sink_adjustment_view(item));
    }
    views
}

pub fn sum_default_sink_adjustment_view(view: &DefaultSinkAdjustmentView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_amount)
}
