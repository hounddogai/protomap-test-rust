use super::*;

use acme_kit::text;

/// How screens and exports show a plan cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanCycleView {
    pub code: String,
    pub title_text: String,
    pub pinned: String,
    pub checksum: String,
    pub source: String,
    pub host: String,
    pub minimum: String,
    pub currency: String,
    pub label: String,
    pub owner_id: String,
    pub status_label: String,
}

pub fn to_plan_cycle_view(item: &PlanCycle) -> PlanCycleView {
    PlanCycleView {
        code: item.code.clone(),
        title_text: item.title.clone(),
        pinned: format!("{}", item.pinned),
        checksum: item.checksum.clone(),
        source: item.source.clone(),
        host: text::truncate(&item.host, 40),
        minimum: format!("{:.2}", item.minimum),
        currency: item.currency.clone(),
        label: item.label.clone(),
        owner_id: text::truncate(&item.owner_id, 64),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_plan_cycle_views(items: &[PlanCycle]) -> Vec<PlanCycleView> {
    let mut views: Vec<PlanCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_plan_cycle_view(item));
    }
    views
}

pub fn sum_plan_cycle_view(view: &PlanCycleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    Ok(parsed_minimum)
}
