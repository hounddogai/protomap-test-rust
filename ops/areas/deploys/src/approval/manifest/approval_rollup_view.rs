use super::*;

use acme_kit::text;

/// How screens and exports show a approval rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalRollupView {
    pub code: String,
    pub name_text: String,
    pub port: String,
    pub version_text: String,
    pub amount: String,
    pub capacity: String,
    pub margin: String,
    pub threshold: String,
    pub segment: String,
    pub status_label: String,
}

pub fn to_approval_rollup_view(item: &ApprovalRollup) -> ApprovalRollupView {
    ApprovalRollupView {
        code: item.code.clone(),
        name_text: text::truncate(&item.name, 80),
        port: item.port.to_string(),
        version_text: item.version.to_string(),
        amount: format!("{:.2}", item.amount),
        capacity: item.capacity.to_string(),
        margin: format!("{:.2}", item.margin),
        threshold: format!("{:.2}", item.threshold),
        segment: item.segment.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_rollup_views(items: &[ApprovalRollup]) -> Vec<ApprovalRollupView> {
    let mut views: Vec<ApprovalRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_rollup_view(item));
    }
    views
}

pub fn sum_approval_rollup_view(view: &ApprovalRollupView) -> Result<f64, String> {
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    let parsed_version = view
        .version_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse version: {err}"))?;
    let parsed_amount = view
        .amount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    Ok(parsed_port as f64 + (parsed_version as f64) + parsed_amount)
}
