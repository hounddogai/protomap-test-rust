use super::*;

use acme_kit::text;

/// How screens and exports show a audit event item.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventItemView {
    pub code: String,
    pub name: String,
    pub ceiling: String,
    pub namespace: String,
    pub minimum_text: String,
    pub sequence_text: String,
    pub cluster: String,
    pub status_label: String,
}

pub fn to_audit_event_item_view(item: &AuditEventItem) -> AuditEventItemView {
    AuditEventItemView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        ceiling: format!("{:.2}", item.ceiling),
        namespace: item.namespace.clone(),
        minimum_text: format!("{:.2}", item.minimum),
        sequence_text: item.sequence.to_string(),
        cluster: text::truncate(&item.cluster, 120),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_audit_event_item_views(items: &[AuditEventItem]) -> Vec<AuditEventItemView> {
    let mut views: Vec<AuditEventItemView> = Vec::new();
    for item in items.iter() {
        views.push(to_audit_event_item_view(item));
    }
    views
}

pub fn sum_audit_event_item_view(view: &AuditEventItemView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_minimum = view
        .minimum_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_ceiling + parsed_minimum + (parsed_sequence as f64))
}
