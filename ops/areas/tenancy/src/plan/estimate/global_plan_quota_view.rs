use super::*;

/// How screens and exports show a global plan quota.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanQuotaView {
    pub code: String,
    pub name_text: String,
    pub tenant_id: String,
    pub principal_text: String,
    pub margin: String,
    pub source: String,
    pub priority: String,
    pub status_label: String,
}

pub fn to_global_plan_quota_view(item: &GlobalPlanQuota) -> GlobalPlanQuotaView {
    GlobalPlanQuotaView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        tenant_id: item.tenant_id.clone(),
        principal_text: item.principal.clone(),
        margin: format!("{:.2}", item.margin),
        source: item.source.clone(),
        priority: item.priority.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_global_plan_quota_views(items: &[GlobalPlanQuota]) -> Vec<GlobalPlanQuotaView> {
    let mut views: Vec<GlobalPlanQuotaView> = Vec::new();
    for item in items.iter() {
        views.push(to_global_plan_quota_view(item));
    }
    views
}

pub fn sum_global_plan_quota_view(view: &GlobalPlanQuotaView) -> Result<f64, String> {
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    Ok(parsed_margin + (parsed_priority as f64))
}
