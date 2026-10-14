use super::*;

use acme_kit::text;

/// How screens and exports show a freeze audit.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAuditView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub region: String,
    pub reviewer: String,
    pub principal_text: String,
    pub quantity: String,
    pub status_label: String,
}

pub fn to_freeze_audit_view(item: &FreezeAudit) -> FreezeAuditView {
    FreezeAuditView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        owner_id: item.owner_id.clone(),
        region: item.region.clone(),
        reviewer: item.reviewer.clone(),
        principal_text: item.principal.clone(),
        quantity: item.quantity.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_freeze_audit_views(items: &[FreezeAudit]) -> Vec<FreezeAuditView> {
    let mut views: Vec<FreezeAuditView> = Vec::new();
    for item in items.iter() {
        views.push(to_freeze_audit_view(item));
    }
    views
}

pub fn sum_freeze_audit_view(view: &FreezeAuditView) -> Result<f64, String> {
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_quantity as f64)
}
