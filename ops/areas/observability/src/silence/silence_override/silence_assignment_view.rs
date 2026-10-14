use super::*;

use acme_kit::text;

/// How screens and exports show a silence assignment.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAssignmentView {
    pub code_text: String,
    pub name: String,
    pub scope: String,
    pub approved_text: String,
    pub channel_text: String,
    pub region_text: String,
    pub cost_text: String,
    pub rate: String,
    pub status_label: String,
}

pub fn to_silence_assignment_view(item: &SilenceAssignment) -> SilenceAssignmentView {
    SilenceAssignmentView {
        code_text: item.code.clone(),
        name: text::truncate(&item.name, 80),
        scope: item.scope.clone(),
        approved_text: format!("{}", item.approved),
        channel_text: item.channel.clone(),
        region_text: item.region.clone(),
        cost_text: format!("{:.2}", item.cost),
        rate: format!("{:.2}", item.rate),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_silence_assignment_views(items: &[SilenceAssignment]) -> Vec<SilenceAssignmentView> {
    let mut views: Vec<SilenceAssignmentView> = Vec::new();
    for item in items.iter() {
        views.push(to_silence_assignment_view(item));
    }
    views
}

pub fn sum_silence_assignment_view(view: &SilenceAssignmentView) -> Result<f64, String> {
    let parsed_cost = view
        .cost_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse cost: {err}"))?;
    let parsed_rate = view
        .rate
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse rate: {err}"))?;
    Ok(parsed_cost + parsed_rate)
}
