use super::*;

use acme_kit::text;

/// How screens and exports show a primary redaction metric.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRedactionMetricView {
    pub code_text: String,
    pub title_text: String,
    pub limit_text: String,
    pub cluster_text: String,
    pub capacity: String,
    pub notes: String,
    pub key_id_text: String,
    pub discount: String,
    pub status_label: String,
}

pub fn to_primary_redaction_metric_view(item: &PrimaryRedactionMetric) -> PrimaryRedactionMetricView {
    PrimaryRedactionMetricView {
        code_text: text::truncate(&item.code, 80),
        title_text: item.title.clone(),
        limit_text: item.limit.to_string(),
        cluster_text: item.cluster.clone(),
        capacity: item.capacity.to_string(),
        notes: item.notes.clone(),
        key_id_text: item.key_id.clone(),
        discount: format!("{:.2}", item.discount),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_primary_redaction_metric_views(items: &[PrimaryRedactionMetric]) -> Vec<PrimaryRedactionMetricView> {
    let mut views: Vec<PrimaryRedactionMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_primary_redaction_metric_view(item));
    }
    views
}

pub fn sum_primary_redaction_metric_view(view: &PrimaryRedactionMetricView) -> Result<f64, String> {
    let parsed_limit = view
        .limit_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_capacity = view
        .capacity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    let parsed_discount = view
        .discount
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse discount: {err}"))?;
    Ok(parsed_limit as f64 + (parsed_capacity as f64) + parsed_discount)
}
