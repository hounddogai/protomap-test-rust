use super::*;

use acme_kit::text;

/// How screens and exports show a capped dashboard exception.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedDashboardExceptionView {
    pub code: String,
    pub label: String,
    pub ceiling: String,
    pub starts_at: String,
    pub checksum: String,
    pub country_text: String,
    pub duration_minutes: String,
    pub name: String,
    pub external_ref: String,
    pub status_label: String,
}

pub fn to_capped_dashboard_exception_view(item: &CappedDashboardException) -> CappedDashboardExceptionView {
    CappedDashboardExceptionView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        ceiling: format!("{:.2}", item.ceiling),
        starts_at: item.starts_at.to_string(),
        checksum: item.checksum.clone(),
        country_text: item.country.clone(),
        duration_minutes: item.duration_minutes.to_string(),
        name: text::truncate(&item.name, 64),
        external_ref: text::truncate(&item.external_ref, 80),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_dashboard_exception_views(items: &[CappedDashboardException]) -> Vec<CappedDashboardExceptionView> {
    let mut views: Vec<CappedDashboardExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_dashboard_exception_view(item));
    }
    views
}

pub fn sum_capped_dashboard_exception_view(view: &CappedDashboardExceptionView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_starts_at = view
        .starts_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse starts at: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    Ok(parsed_ceiling + (parsed_starts_at as f64) + (parsed_duration_minutes as f64))
}
