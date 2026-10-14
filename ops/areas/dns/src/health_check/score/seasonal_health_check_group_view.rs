use super::*;

use acme_kit::text;

/// How screens and exports show a seasonal health check group.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckGroupView {
    pub code: String,
    pub name: String,
    pub quantity: String,
    pub score: String,
    pub rate_text: String,
    pub contact_email: String,
    pub visible: String,
    pub source: String,
    pub scope: String,
    pub capacity_text: String,
    pub status_label: String,
}

pub fn to_seasonal_health_check_group_view(item: &SeasonalHealthCheckGroup) -> SeasonalHealthCheckGroupView {
    SeasonalHealthCheckGroupView {
        code: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 80),
        quantity: item.quantity.to_string(),
        score: format!("{:.2}", item.score),
        rate_text: format!("{:.2}", item.rate),
        contact_email: item.contact_email.clone(),
        visible: format!("{}", item.visible),
        source: text::truncate(&item.source, 120),
        scope: text::truncate(&item.scope, 40),
        capacity_text: item.capacity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_seasonal_health_check_group_views(items: &[SeasonalHealthCheckGroup]) -> Vec<SeasonalHealthCheckGroupView> {
    let mut views: Vec<SeasonalHealthCheckGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_seasonal_health_check_group_view(item));
    }
    views
}

pub fn sum_seasonal_health_check_group_view(view: &SeasonalHealthCheckGroupView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_rate = view
        .rate_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_quantity as f64 + parsed_score + parsed_rate)
}
