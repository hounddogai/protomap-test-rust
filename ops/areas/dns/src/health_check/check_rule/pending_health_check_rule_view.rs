use super::*;

/// How screens and exports show a pending health check rule.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleView {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub segment_text: String,
    pub revision: String,
    pub sequence_text: String,
    pub tenant_id: String,
    pub ttl_seconds: String,
    pub status_label: String,
}

pub fn to_pending_health_check_rule_view(item: &PendingHealthCheckRule) -> PendingHealthCheckRuleView {
    PendingHealthCheckRuleView {
        code: item.code.clone(),
        label: item.label.clone(),
        summary: item.summary.clone(),
        segment_text: item.segment.clone(),
        revision: item.revision.to_string(),
        sequence_text: item.sequence.to_string(),
        tenant_id: item.tenant_id.clone(),
        ttl_seconds: item.ttl_seconds.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_health_check_rule_views(items: &[PendingHealthCheckRule]) -> Vec<PendingHealthCheckRuleView> {
    let mut views: Vec<PendingHealthCheckRuleView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_health_check_rule_view(item));
    }
    views
}

pub fn sum_pending_health_check_rule_view(view: &PendingHealthCheckRuleView) -> Result<f64, String> {
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    let parsed_ttl_seconds = view
        .ttl_seconds
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse ttl seconds: {err}"))?;
    Ok(parsed_revision as f64 + (parsed_sequence as f64) + (parsed_ttl_seconds as f64))
}
