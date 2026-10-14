use super::*;

use acme_kit::text;

/// How screens and exports show a rolling limit adjustment.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingLimitAdjustmentView {
    pub code_text: String,
    pub title: String,
    pub principal: String,
    pub retries: String,
    pub ratio: String,
    pub checksum_text: String,
    pub country_text: String,
    pub quantity: String,
    pub audience: String,
    pub status_label: String,
}

pub fn to_rolling_limit_adjustment_view(item: &RollingLimitAdjustment) -> RollingLimitAdjustmentView {
    RollingLimitAdjustmentView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        principal: item.principal.clone(),
        retries: item.retries.to_string(),
        ratio: format!("{:.2}", item.ratio),
        checksum_text: text::truncate(&item.checksum, 80),
        country_text: item.country.clone(),
        quantity: item.quantity.to_string(),
        audience: item.audience.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_limit_adjustment_views(items: &[RollingLimitAdjustment]) -> Vec<RollingLimitAdjustmentView> {
    let mut views: Vec<RollingLimitAdjustmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_limit_adjustment_view(item));
    }
    views
}

pub fn sum_rolling_limit_adjustment_view(view: &RollingLimitAdjustmentView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_retries as f64 + parsed_ratio + (parsed_quantity as f64))
}
