use super::*;

/// How screens and exports show a strict tenant allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAllocationView {
    pub code: String,
    pub name: String,
    pub revision: String,
    pub unit: String,
    pub segment: String,
    pub threshold: String,
    pub minimum: String,
    pub budget_text: String,
    pub version: String,
    pub ends_at: String,
    pub contact_email: String,
    pub status_label: String,
}

pub fn to_strict_tenant_allocation_view(item: &StrictTenantAllocation) -> StrictTenantAllocationView {
    StrictTenantAllocationView {
        code: item.code.clone(),
        name: item.name.clone(),
        revision: item.revision.to_string(),
        unit: item.unit.clone(),
        segment: item.segment.clone(),
        threshold: format!("{:.2}", item.threshold),
        minimum: format!("{:.2}", item.minimum),
        budget_text: format!("{:.2}", item.budget),
        version: item.version.to_string(),
        ends_at: item.ends_at.to_string(),
        contact_email: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_strict_tenant_allocation_views(items: &[StrictTenantAllocation]) -> Vec<StrictTenantAllocationView> {
    let mut views: Vec<StrictTenantAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_strict_tenant_allocation_view(item));
    }
    views
}

pub fn sum_strict_tenant_allocation_view(view: &StrictTenantAllocationView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_threshold = view
        .threshold
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse threshold: {err}"))?;
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_revision as f64 + parsed_threshold + parsed_minimum)
}
