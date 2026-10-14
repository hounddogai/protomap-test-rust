use super::*;

/// How screens and exports show a ttl policy metric.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMetricView {
    pub code_text: String,
    pub name: String,
    pub batch_size: String,
    pub contact_email: String,
    pub floor_value: String,
    pub issuer: String,
    pub principal: String,
    pub pinned: String,
    pub rotation_days: String,
    pub source_text: String,
    pub status_label: String,
}

pub fn to_ttl_policy_metric_view(item: &TtlPolicyMetric) -> TtlPolicyMetricView {
    TtlPolicyMetricView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        batch_size: item.batch_size.to_string(),
        contact_email: item.contact_email.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        issuer: item.issuer.clone(),
        principal: item.principal.clone(),
        pinned: format!("{}", item.pinned),
        rotation_days: item.rotation_days.to_string(),
        source_text: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_ttl_policy_metric_views(items: &[TtlPolicyMetric]) -> Vec<TtlPolicyMetricView> {
    let mut views: Vec<TtlPolicyMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_ttl_policy_metric_view(item));
    }
    views
}

pub fn sum_ttl_policy_metric_view(view: &TtlPolicyMetricView) -> Result<f64, String> {
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_batch_size as f64 + parsed_floor_value + (parsed_rotation_days as f64))
}
