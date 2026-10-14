use super::*;

/// How screens and exports show a scheduled dashboard quota.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardQuotaView {
    pub code_text: String,
    pub name: String,
    pub ratio: String,
    pub duration_minutes: String,
    pub amount: String,
    pub audience: String,
    pub key_id: String,
    pub ends_at_text: String,
    pub cost_text: String,
    pub scope: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_scheduled_dashboard_quota_view(item: &ScheduledDashboardQuota) -> ScheduledDashboardQuotaView {
    ScheduledDashboardQuotaView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        ratio: format!("{:.2}", item.ratio),
        duration_minutes: item.duration_minutes.to_string(),
        amount: format!("{:.2}", item.amount),
        audience: item.audience.clone(),
        key_id: item.key_id.clone(),
        ends_at_text: item.ends_at.to_string(),
        cost_text: format!("{:.2}", item.cost),
        scope: item.scope.clone(),
        cluster: item.cluster.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_scheduled_dashboard_quota_views(items: &[ScheduledDashboardQuota]) -> Vec<ScheduledDashboardQuotaView> {
    let mut views: Vec<ScheduledDashboardQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_scheduled_dashboard_quota_view(item));
    }
    views
}

pub fn sum_scheduled_dashboard_quota_view(view: &ScheduledDashboardQuotaView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_duration_minutes = view
        .duration_minutes
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse duration minutes: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_ratio + (parsed_duration_minutes as f64) + parsed_amount)
}
