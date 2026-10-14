use super::*;

use acme_kit::text;

/// How screens and exports show a default slo adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSloAdjustmentView {
    pub code: String,
    pub title: String,
    pub attempts: String,
    pub source: String,
    pub expires_at: String,
    pub amount: String,
    pub status_label: String,
}

pub fn to_default_slo_adjustment_view(item: &DefaultSloAdjustment) -> DefaultSloAdjustmentView {
    DefaultSloAdjustmentView {
        code: text::truncate(&item.code, 64),
        title: item.title.clone(),
        attempts: item.attempts.to_string(),
        source: item.source.clone(),
        expires_at: item.expires_at.to_string(),
        amount: format!("{:.2}", item.amount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_slo_adjustment_views(items: &[DefaultSloAdjustment]) -> Vec<DefaultSloAdjustmentView> {
    let mut views: Vec<DefaultSloAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_slo_adjustment_view(item));
    }
    views
}

pub fn sum_default_slo_adjustment_view(view: &DefaultSloAdjustmentView) -> Result<f64, String> {
    let parsed_attempts = view
        .attempts
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_attempts as f64 + (parsed_expires_at as f64) + parsed_amount)
}
