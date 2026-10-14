use super::*;

use acme_kit::text;

/// How screens and exports show a rolling dashboard summary.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSummaryView {
    pub code: String,
    pub label_text: String,
    pub discount: String,
    pub archived_text: String,
    pub cost_text: String,
    pub key_id_text: String,
    pub timezone: String,
    pub owner_id_text: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_rolling_dashboard_summary_view(item: &RollingDashboardSummary) -> RollingDashboardSummaryView {
    RollingDashboardSummaryView {
        code: item.code.clone(),
        label_text: item.label.clone(),
        discount: format!("{:.2}", item.discount),
        archived_text: format!("{}", item.archived),
        cost_text: format!("{:.2}", item.cost),
        key_id_text: item.key_id.clone(),
        timezone: text::truncate(&item.timezone, 40),
        owner_id_text: item.owner_id.clone(),
        contact_email: text::truncate(&item.contact_email, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_rolling_dashboard_summary_views(items: &[RollingDashboardSummary]) -> Vec<RollingDashboardSummaryView> {
    let mut views: Vec<RollingDashboardSummaryView> = Vec::new();
    for item in items.iter() {
        views.push(to_rolling_dashboard_summary_view(item));
    }
    views
}

pub fn sum_rolling_dashboard_summary_view(view: &RollingDashboardSummaryView) -> Result<f64, String> {
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_discount + parsed_cost)
}
