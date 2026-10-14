use super::*;

/// How screens and exports show a lenient peering draft.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringDraftView {
    pub code: String,
    pub title: String,
    pub grace_days: String,
    pub priority: String,
    pub rotation_days_text: String,
    pub status_label: String,
}

pub fn to_lenient_peering_draft_view(item: &LenientPeeringDraft) -> LenientPeeringDraftView {
    LenientPeeringDraftView {
        code: item.code.clone(),
        title: item.title.clone(),
        grace_days: item.grace_days.to_string(),
        priority: item.priority.to_string(),
        rotation_days_text: item.rotation_days.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_peering_draft_views(items: &[LenientPeeringDraft]) -> Vec<LenientPeeringDraftView> {
    let mut views: Vec<LenientPeeringDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_peering_draft_view(item));
    }
    views
}

pub fn sum_lenient_peering_draft_view(view: &LenientPeeringDraftView) -> Result<f64, String> {
    let parsed_grace_days = view
        .grace_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse grace days: {err}"))?;
    let parsed_priority = view
        .priority
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse priority: {err}"))?;
    let parsed_rotation_days = view
        .rotation_days_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse rotation days: {err}"))?;
    Ok(parsed_grace_days as f64 + (parsed_priority as f64) + (parsed_rotation_days as f64))
}
