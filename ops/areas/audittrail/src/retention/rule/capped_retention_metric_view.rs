use super::*;

use acme_kit::text;

/// How screens and exports show a capped retention metric.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedRetentionMetricView {
    pub code: String,
    pub label: String,
    pub tenant_id_text: String,
    pub region_text: String,
    pub limit_text: String,
    pub status_label: String,
}

pub fn to_capped_retention_metric_view(item: &CappedRetentionMetric) -> CappedRetentionMetricView {
    CappedRetentionMetricView {
        code: item.code.clone(),
        label: text::truncate(&item.label, 64),
        tenant_id_text: item.tenant_id.clone(),
        region_text: item.region.clone(),
        limit_text: item.limit.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_capped_retention_metric_views(items: &[CappedRetentionMetric]) -> Vec<CappedRetentionMetricView> {
    let mut views: Vec<CappedRetentionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_capped_retention_metric_view(item));
    }
    views
}

pub fn sum_capped_retention_metric_view(view: &CappedRetentionMetricView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    Ok(parsed_limit as f64)
}
