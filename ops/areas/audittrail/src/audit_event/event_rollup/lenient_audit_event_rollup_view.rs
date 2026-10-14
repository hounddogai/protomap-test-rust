use super::*;

use acme_kit::text;

/// How screens and exports show a lenient audit event rollup.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientAuditEventRollupView {
    pub code_text: String,
    pub name: String,
    pub title: String,
    pub country: String,
    pub approved_text: String,
    pub score: String,
    pub ceiling: String,
    pub checksum_text: String,
    pub floor_value: String,
    pub namespace: String,
    pub status_label: String,
}

pub fn to_lenient_audit_event_rollup_view(item: &LenientAuditEventRollup) -> LenientAuditEventRollupView {
    LenientAuditEventRollupView {
        code_text: item.code.clone(),
        name: item.name.clone(),
        title: item.title.clone(),
        country: text::truncate(&item.country, 64),
        approved_text: format!("{}", item.approved),
        score: format!("{:.2}", item.score),
        ceiling: format!("{:.2}", item.ceiling),
        checksum_text: item.checksum.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        namespace: item.namespace.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_audit_event_rollup_views(items: &[LenientAuditEventRollup]) -> Vec<LenientAuditEventRollupView> {
    let mut views: Vec<LenientAuditEventRollupView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_audit_event_rollup_view(item));
    }
    views
}

pub fn sum_lenient_audit_event_rollup_view(view: &LenientAuditEventRollupView) -> Result<f64, String> {
    let parsed_score = view
        .score
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse score: {err}"))?;
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_score + parsed_ceiling + parsed_floor_value)
}
