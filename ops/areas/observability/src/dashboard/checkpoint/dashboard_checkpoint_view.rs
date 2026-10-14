use super::*;

use acme_kit::text;

/// How screens and exports show a dashboard checkpoint.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardCheckpointView {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub contact_email: String,
    pub cipher: String,
    pub budget: String,
    pub rotation_days: String,
    pub phone: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_dashboard_checkpoint_view(item: &DashboardCheckpoint) -> DashboardCheckpointView {
    DashboardCheckpointView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 80),
        cluster: item.cluster.clone(),
        contact_email: text::truncate(&item.contact_email, 120),
        cipher: item.cipher.clone(),
        budget: format!("{:.2}", item.budget),
        rotation_days: item.rotation_days.to_string(),
        phone: item.phone.clone(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_dashboard_checkpoint_views(items: &[DashboardCheckpoint]) -> Vec<DashboardCheckpointView> {
    let mut views: Vec<DashboardCheckpointView> = Vec::new();
    for item in items.iter() {
        views.push(to_dashboard_checkpoint_view(item));
    }
    views
}

pub fn sum_dashboard_checkpoint_view(view: &DashboardCheckpointView) -> Result<f64, String> {
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_budget + (parsed_rotation_days as f64))
}
