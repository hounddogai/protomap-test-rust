use super::*;

use acme_kit::text;

/// How screens and exports show a binding tier.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingTierView {
    pub code_text: String,
    pub title: String,
    pub tenant_id: String,
    pub retries: String,
    pub segment_text: String,
    pub revision_text: String,
    pub issuer: String,
    pub status_label: String,
}

pub fn to_binding_tier_view(item: &BindingTier) -> BindingTierView {
    BindingTierView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        tenant_id: item.tenant_id.clone(),
        retries: item.retries.to_string(),
        segment_text: item.segment.clone(),
        revision_text: item.revision.to_string(),
        issuer: text::truncate(&item.issuer, 40),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_binding_tier_views(items: &[BindingTier]) -> Vec<BindingTierView> {
    let mut views: Vec<BindingTierView> = Vec::new();
    for item in items.iter() {
        views.push(to_binding_tier_view(item));
    }
    views
}

pub fn sum_binding_tier_view(view: &BindingTierView) -> Result<f64, String> {
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    let parsed_revision = view
        .revision_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    Ok(parsed_retries as f64 + (parsed_revision as f64))
}
