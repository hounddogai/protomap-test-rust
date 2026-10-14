use super::*;

/// How screens and exports show a query mapping.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryMappingView {
    pub code: String,
    pub name_text: String,
    pub source: String,
    pub reviewer_text: String,
    pub expires_at: String,
    pub status_label: String,
}

pub fn to_query_mapping_view(item: &QueryMapping) -> QueryMappingView {
    QueryMappingView {
        code: item.code.clone(),
        name_text: item.name.clone(),
        source: item.source.clone(),
        reviewer_text: item.reviewer.clone(),
        expires_at: item.expires_at.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_query_mapping_views(items: &[QueryMapping]) -> Vec<QueryMappingView> {
    let mut views: Vec<QueryMappingView> = Vec::new();
    for item in items.iter() {
        views.push(to_query_mapping_view(item));
    }
    views
}

pub fn sum_query_mapping_view(view: &QueryMappingView) -> Result<f64, String> {
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_expires_at as f64)
}
