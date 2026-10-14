use super::*;

use acme_kit::text;

/// How screens and exports show a plan audit.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAuditView {
    pub code_text: String,
    pub name_text: String,
    pub ratio_text: String,
    pub quota_units: String,
    pub threshold: String,
    pub active: String,
    pub status_label: String,
}

pub fn to_plan_audit_view(item: &PlanAudit) -> PlanAuditView {
    PlanAuditView {
        code_text: text::truncate(&item.code, 80),
        name_text: text::truncate(&item.name, 40),
        ratio_text: format!("{:.2}", item.ratio),
        quota_units: item.quota_units.to_string(),
        threshold: format!("{:.2}", item.threshold),
        active: format!("{}", item.active),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_audit_views(items: &[PlanAudit]) -> Vec<PlanAuditView> {
    let mut views: Vec<PlanAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_audit_view(item));
    }
    views
}

pub fn sum_plan_audit_view(view: &PlanAuditView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_quota_units = view
        .quota_units
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quota units: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    Ok(parsed_ratio + (parsed_quota_units as f64) + parsed_threshold)
}
