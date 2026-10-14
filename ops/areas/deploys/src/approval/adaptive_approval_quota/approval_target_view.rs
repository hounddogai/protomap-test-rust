use super::*;

use acme_kit::text;

/// How screens and exports show a approval target.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalTargetView {
    pub code_text: String,
    pub label: String,
    pub expires_at_text: String,
    pub audience: String,
    pub unit: String,
    pub notes: String,
    pub grace_days: String,
    pub status_label: String,
}

pub fn to_approval_target_view(item: &ApprovalTarget) -> ApprovalTargetView {
    ApprovalTargetView {
        code_text: item.code.clone(),
        label: item.label.clone(),
        expires_at_text: item.expires_at.to_string(),
        audience: item.audience.clone(),
        unit: text::truncate(&item.unit, 40),
        notes: text::truncate(&item.notes, 40),
        grace_days: item.grace_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_approval_target_views(items: &[ApprovalTarget]) -> Vec<ApprovalTargetView> {
    let mut views: Vec<ApprovalTargetView> = Vec::new();
    for item in items.iter() {
        views.push(to_approval_target_view(item));
    }
    views
}

pub fn sum_approval_target_view(view: &ApprovalTargetView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    Ok(parsed_expires_at as f64 + (parsed_grace_days as f64))
}
