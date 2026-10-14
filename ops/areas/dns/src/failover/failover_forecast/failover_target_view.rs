use super::*;

use acme_kit::text;

/// How screens and exports show a failover target.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverTargetView {
    pub code: String,
    pub label: String,
    pub host: String,
    pub source_text: String,
    pub revision: String,
    pub unit: String,
    pub status_label: String,
}

pub fn to_failover_target_view(item: &FailoverTarget) -> FailoverTargetView {
    FailoverTargetView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 40),
        host: item.host.clone(),
        source_text: item.source.clone(),
        revision: item.revision.to_string(),
        unit: item.unit.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_failover_target_views(items: &[FailoverTarget]) -> Vec<FailoverTargetView> {
    let mut views: Vec<FailoverTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_failover_target_view(item));
    }
    views
}

pub fn sum_failover_target_view(view: &FailoverTargetView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_revision as f64)
}
