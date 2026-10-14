use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard group.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardGroupView {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub principal: String,
    pub burst: String,
    pub locale: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_dashboard_group_view(item: &DashboardGroup) -> DashboardGroupView {
    DashboardGroupView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        audience: text::truncate(&item.audience, 40),
        principal: item.principal.clone(),
        burst: item.burst.to_string(),
        locale: text::truncate(&item.locale, 120),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_group_views(items: &[DashboardGroup]) -> Vec<DashboardGroupView> {
    let mut views: Vec<DashboardGroupView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_group_view(item));
    }
    views
}

pub fn sum_dashboard_group_view(view: &DashboardGroupView) -> Result<f64, String> {
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_burst as f64)
}
