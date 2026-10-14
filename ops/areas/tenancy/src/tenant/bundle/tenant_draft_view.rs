use super::*;

/// How screens and exports show a tenant draft.
#[derive(Debug, Clone, PartialEq)]
pub struct TenantDraftView {
    pub code: String,
    pub name: String,
    pub ratio: String,
    pub region: String,
    pub ceiling_text: String,
    pub burst: String,
    pub quota_units: String,
    pub status_label: String,
}

pub fn to_tenant_draft_view(item: &TenantDraft) -> TenantDraftView {
    TenantDraftView {
        code: item.code.clone(),
        name: item.name.clone(),
        ratio: format!("{:.2}", item.ratio),
        region: item.region.clone(),
        ceiling_text: format!("{:.2}", item.ceiling),
        burst: item.burst.to_string(),
        quota_units: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tenant_draft_views(items: &[TenantDraft]) -> Vec<TenantDraftView> {
    let mut views: Vec<TenantDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_tenant_draft_view(item));
    }
    views
}

pub fn sum_tenant_draft_view(view: &TenantDraftView) -> Result<f64, String> {
    let parsed_ratio = view
        .ratio
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ratio: {err}"))?;
    let parsed_ceiling = view
        .ceiling_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_burst = view
        .burst
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse burst: {err}"))?;
    Ok(parsed_ratio + parsed_ceiling + (parsed_burst as f64))
}
