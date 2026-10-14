use super::*;

use acme_kit::text;

/// How screens and exports show a primary dashboard template.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDashboardTemplateView {
    pub code: String,
    pub title_text: String,
    pub ratio: String,
    pub quantity_text: String,
    pub key_id: String,
    pub ttl_seconds_text: String,
    pub name: String,
    pub issuer: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_primary_dashboard_template_view(item: &PrimaryDashboardTemplate) -> PrimaryDashboardTemplateView {
    PrimaryDashboardTemplateView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        ratio: format!("{:.2}", item.ratio),
        quantity_text: item.quantity.to_string(),
        key_id: text::truncate(&item.key_id, 80),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        name: item.name.clone(),
        issuer: item.issuer.clone(),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_dashboard_template_views(items: &[PrimaryDashboardTemplate]) -> Vec<PrimaryDashboardTemplateView> {
    let mut views: Vec<PrimaryDashboardTemplateView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_dashboard_template_view(item));
    }
    views
}

pub fn sum_primary_dashboard_template_view(view: &PrimaryDashboardTemplateView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quantity = view
        .quantity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_ratio + (parsed_quantity as f64) + (parsed_ttl_seconds as f64))
}
