use super::*;

use acme_kit::text;

/// How screens and exports show a failover allocation.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverAllocationView {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub revision: String,
    pub expires_at_text: String,
    pub region: String,
    pub status_label: String,
}

pub fn to_failover_allocation_view(item: &FailoverAllocation) -> FailoverAllocationView {
    FailoverAllocationView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        currency: item.currency.clone(),
        revision: item.revision.to_string(),
        expires_at_text: item.expires_at.to_string(),
        region: item.region.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_allocation_views(items: &[FailoverAllocation]) -> Vec<FailoverAllocationView> {
    let mut views: Vec<FailoverAllocationView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_allocation_view(item));
    }
    views
}

pub fn sum_failover_allocation_view(view: &FailoverAllocationView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_expires_at as f64))
}
