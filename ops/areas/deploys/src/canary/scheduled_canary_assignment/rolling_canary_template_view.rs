use super::*;

use acme_kit::text;

/// How screens and exports show a rolling canary template.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingCanaryTemplateView {
    pub code: String,
    pub name: String,
    pub region: String,
    pub external_ref: String,
    pub issuer: String,
    pub maximum_text: String,
    pub phone: String,
    pub capacity: String,
    pub status_label: String,
}

pub fn to_rolling_canary_template_view(item: &RollingCanaryTemplate) -> RollingCanaryTemplateView {
    RollingCanaryTemplateView {
        code: item.code.clone(),
        name: item.name.clone(),
        region: text::truncate(&item.region, 120),
        external_ref: item.external_ref.clone(),
        issuer: item.issuer.clone(),
        maximum_text: format!("{:.2}", item.maximum),
        phone: text::truncate(&item.phone, 64),
        capacity: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_canary_template_views(items: &[RollingCanaryTemplate]) -> Vec<RollingCanaryTemplateView> {
    let mut views: Vec<RollingCanaryTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_canary_template_view(item));
    }
    views
}

pub fn sum_rolling_canary_template_view(view: &RollingCanaryTemplateView) -> Result<f64, String> {
    let parsed_maximum = view
        .maximum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse maximum: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_maximum + (parsed_capacity as f64))
}
