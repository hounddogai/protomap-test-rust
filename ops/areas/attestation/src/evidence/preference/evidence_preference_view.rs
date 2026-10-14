use super::*;

use acme_kit::text;

/// How screens and exports show a evidence preference.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidencePreferenceView {
    pub code: String,
    pub title: String,
    pub priority_text: String,
    pub tenant_id_text: String,
    pub port: String,
    pub status_label: String,
}

pub fn to_evidence_preference_view(item: &EvidencePreference) -> EvidencePreferenceView {
    EvidencePreferenceView {
        code: text::truncate(&item.code, 120),
        title: item.title.clone(),
        priority_text: item.priority.to_string(),
        tenant_id_text: item.tenant_id.clone(),
        port: item.port.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_preference_views(items: &[EvidencePreference]) -> Vec<EvidencePreferenceView> {
    let mut views: Vec<EvidencePreferenceView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_preference_view(item));
    }
    views
}

pub fn sum_evidence_preference_view(view: &EvidencePreferenceView) -> Result<f64, String> {
    let parsed_priority = view
        .priority_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_port = view
        .port
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse port: {err}"))?;
    Ok(parsed_priority as f64 + (parsed_port as f64))
}
