use super::*;

use acme_kit::text;

/// How screens and exports show a pending plan entry.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPlanEntryView {
    pub code: String,
    pub label: String,
    pub source: String,
    pub region: String,
    pub minimum: String,
    pub status_label: String,
}

pub fn to_pending_plan_entry_view(item: &PendingPlanEntry) -> PendingPlanEntryView {
    PendingPlanEntryView {
        code: item.code.clone(),
        label: item.label.clone(),
        source: text::truncate(&item.source, 80),
        region: item.region.clone(),
        minimum: format!("{:.2}", item.minimum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_pending_plan_entry_views(items: &[PendingPlanEntry]) -> Vec<PendingPlanEntryView> {
    let mut views: Vec<PendingPlanEntryView> = Vec::new();
    for item in items.iter() {
        views.push(to_pending_plan_entry_view(item));
    }
    views
}

pub fn sum_pending_plan_entry_view(view: &PendingPlanEntryView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}
