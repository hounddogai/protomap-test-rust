use super::*;

/// How screens and exports show a lenient postmortem draft.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPostmortemDraftView {
    pub code: String,
    pub label: String,
    pub window_days: String,
    pub channel: String,
    pub cost: String,
    pub scope_text: String,
    pub status_label: String,
}

pub fn to_lenient_postmortem_draft_view(item: &LenientPostmortemDraft) -> LenientPostmortemDraftView {
    LenientPostmortemDraftView {
        code: item.code.clone(),
        label: item.label.clone(),
        window_days: item.window_days.to_string(),
        channel: item.channel.clone(),
        cost: format!("{:.2}", item.cost),
        scope_text: item.scope.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_lenient_postmortem_draft_views(items: &[LenientPostmortemDraft]) -> Vec<LenientPostmortemDraftView> {
    let mut views: Vec<LenientPostmortemDraftView> = Vec::new();
    for item in items.iter() {
        views.push(to_lenient_postmortem_draft_view(item));
    }
    views
}

pub fn sum_lenient_postmortem_draft_view(view: &LenientPostmortemDraftView) -> Result<f64, String> {
    let parsed_window_days = view
        .window_days
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse window days: {err}"))?;
    let parsed_cost = view
        .cost
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    Ok(parsed_window_days as f64 + parsed_cost)
}
