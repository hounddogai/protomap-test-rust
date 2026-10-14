use super::*;

use acme_kit::text;

/// How screens and exports show a probe draft.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeDraftView {
    pub code: String,
    pub label_text: String,
    pub ceiling: String,
    pub batch_size: String,
    pub source: String,
    pub budget: String,
    pub version_text: String,
    pub cost_text: String,
    pub tenant_id_text: String,
    pub status_label: String,
}

pub fn to_probe_draft_view(item: &ProbeDraft) -> ProbeDraftView {
    ProbeDraftView {
        code: text::truncate(&item.code, 120),
        label_text: text::truncate(&item.label, 120),
        ceiling: format!("{:.2}", item.ceiling),
        batch_size: item.batch_size.to_string(),
        source: item.source.clone(),
        budget: format!("{:.2}", item.budget),
        version_text: item.version.to_string(),
        cost_text: format!("{:.2}", item.cost),
        tenant_id_text: item.tenant_id.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_probe_draft_views(items: &[ProbeDraft]) -> Vec<ProbeDraftView> {
    let mut views: Vec<ProbeDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_probe_draft_view(item));
    }
    views
}

pub fn sum_probe_draft_view(view: &ProbeDraftView) -> Result<f64, String> {
    let parsed_ceiling = view
        .ceiling
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse ceiling: {err}"))?;
    let parsed_batch_size = view
        .batch_size
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse batch size: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    Ok(parsed_ceiling + (parsed_batch_size as f64) + parsed_budget)
}
