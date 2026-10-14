use super::*;

use acme_kit::text;

/// How screens and exports show a default tenant mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultTenantMappingView {
    pub code_text: String,
    pub title: String,
    pub summary: String,
    pub ttl_seconds: String,
    pub approved: String,
    pub cluster: String,
    pub description: String,
    pub margin: String,
    pub retries: String,
    pub status_label: String,
}

pub fn to_default_tenant_mapping_view(item: &DefaultTenantMapping) -> DefaultTenantMappingView {
    DefaultTenantMappingView {
        code_text: item.code.clone(),
        title: text::truncate(&item.title, 80),
        summary: item.summary.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        approved: format!("{}", item.approved),
        cluster: item.cluster.clone(),
        description: item.description.clone(),
        margin: format!("{:.2}", item.margin),
        retries: item.retries.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_default_tenant_mapping_views(items: &[DefaultTenantMapping]) -> Vec<DefaultTenantMappingView> {
    let mut views: Vec<DefaultTenantMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_default_tenant_mapping_view(item));
    }
    views
}

pub fn sum_default_tenant_mapping_view(view: &DefaultTenantMappingView) -> Result<f64, String> {
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_retries = view
        .retries
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse retries: {err}"))?;
    Ok(parsed_ttl_seconds as f64 + parsed_margin + (parsed_retries as f64))
}
