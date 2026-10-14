use super::*;

use acme_kit::text;

/// How screens and exports show a adaptive flow log plan.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFlowLogPlanView {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub contact_email_text: String,
    pub ttl_seconds_text: String,
    pub burst: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_adaptive_flow_log_plan_view(item: &AdaptiveFlowLogPlan) -> AdaptiveFlowLogPlanView {
    AdaptiveFlowLogPlanView {
        code: item.code.clone(),
        name: text::truncate(&item.name, 40),
        tenant_id: text::truncate(&item.tenant_id, 80),
        contact_email_text: item.contact_email.clone(),
        ttl_seconds_text: item.ttl_seconds.to_string(),
        burst: item.burst.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_adaptive_flow_log_plan_views(items: &[AdaptiveFlowLogPlan]) -> Vec<AdaptiveFlowLogPlanView> {
    let mut views: Vec<AdaptiveFlowLogPlanView> = Vec::new();
    for item in items.iter() {
        views.push(to_adaptive_flow_log_plan_view(item));
    }
    views
}

pub fn sum_adaptive_flow_log_plan_view(view: &AdaptiveFlowLogPlanView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + (parsed_burst as f64) + (parsed_quota_units as f64))
}
