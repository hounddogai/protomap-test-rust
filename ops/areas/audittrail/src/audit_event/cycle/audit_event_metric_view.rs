use super::*;

use acme_kit::text;

/// How screens and exports show a audit event metric.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMetricView {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub cluster: String,
    pub fingerprint: String,
    pub limit: String,
    pub margin: String,
    pub description: String,
    pub quantity: String,
    pub weight_text: String,
    pub status_label: String,
}

pub fn to_audit_event_metric_view(item: &AuditEventMetric) -> AuditEventMetricView {
    AuditEventMetricView {
        code: text::truncate(&item.code, 120),
        name: item.name.clone(),
        owner_id: text::truncate(&item.owner_id, 40),
        cluster: item.cluster.clone(),
        fingerprint: text::truncate(&item.fingerprint, 64),
        limit: item.limit.to_string(),
        margin: format!("{:.2}", item.margin),
        description: item.description.clone(),
        quantity: item.quantity.to_string(),
        weight_text: format!("{:.2}", item.weight),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_metric_views(items: &[AuditEventMetric]) -> Vec<AuditEventMetricView> {
    let mut views: Vec<AuditEventMetricView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_metric_view(item));
    }
    views
}

pub fn sum_audit_event_metric_view(view: &AuditEventMetricView) -> Result<f64, String> {
    let parsed_limit = view
        .limit
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse limit: {err}"))?;
    let parsed_margin = view
        .margin
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse margin: {err}"))?;
    let parsed_quantity = view
        .quantity
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse quantity: {err}"))?;
    Ok(parsed_limit as f64 + parsed_margin + (parsed_quantity as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_audit_event_metric("code-919", "name-401");
        let view = to_audit_event_metric_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_audit_event_metric_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
