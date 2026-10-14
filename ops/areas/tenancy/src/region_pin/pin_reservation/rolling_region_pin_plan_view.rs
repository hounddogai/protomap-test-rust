use super::*;

use acme_kit::text;

/// How screens and exports show a rolling region pin plan.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlanView {
    pub code: String,
    pub title: String,
    pub quota_units: String,
    pub currency: String,
    pub source_text: String,
    pub threshold_text: String,
    pub status_label: String,
}

pub fn to_rolling_region_pin_plan_view(item: &RollingRegionPinPlan) -> RollingRegionPinPlanView {
    RollingRegionPinPlanView {
        code: text::truncate(&item.code, 80),
        title: text::truncate(&item.title, 64),
        quota_units: item.quota_units.to_string(),
        currency: text::truncate(&item.currency, 64),
        source_text: item.source.clone(),
        threshold_text: format!("{:.2}", item.threshold),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_region_pin_plan_views(items: &[RollingRegionPinPlan]) -> Vec<RollingRegionPinPlanView> {
    let mut views: Vec<RollingRegionPinPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_region_pin_plan_view(item));
    }
    views
}

pub fn sum_rolling_region_pin_plan_view(view: &RollingRegionPinPlanView) -> Result<f64, String> {
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_threshold = view
        .threshold_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_quota_units as f64 + parsed_threshold)
}
