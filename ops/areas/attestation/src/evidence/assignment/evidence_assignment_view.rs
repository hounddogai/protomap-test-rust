use super::*;

use acme_kit::text;

/// How screens and exports show a evidence assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAssignmentView {
    pub code: String,
    pub name: String,
    pub updated_at: String,
    pub cost: String,
    pub unit: String,
    pub audience_text: String,
    pub expires_at: String,
    pub source: String,
    pub status_label: String,
}

pub fn to_evidence_assignment_view(item: &EvidenceAssignment) -> EvidenceAssignmentView {
    EvidenceAssignmentView {
        code: text::truncate(&item.code, 80),
        name: item.name.clone(),
        updated_at: item.updated_at.to_string(),
        cost: format!("{:.2}", item.cost),
        unit: item.unit.clone(),
        audience_text: text::truncate(&item.audience, 120),
        expires_at: item.expires_at.to_string(),
        source: item.source.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_evidence_assignment_views(items: &[EvidenceAssignment]) -> Vec<EvidenceAssignmentView> {
    let mut views: Vec<EvidenceAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_evidence_assignment_view(item));
    }
    views
}

pub fn sum_evidence_assignment_view(view: &EvidenceAssignmentView) -> Result<f64, String> {
    let parsed_updated_at = view
        .updated_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse updated at: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_expires_at = view
        .expires_at
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse expires at: {err}"))?;
    Ok(parsed_updated_at as f64 + parsed_cost + (parsed_expires_at as f64))
}
