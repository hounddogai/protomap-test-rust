use super::*;

use acme_kit::text;

/// How screens and exports show a tiered egress cycle.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressCycleView {
    pub code_text: String,
    pub name: String,
    pub minimum: String,
    pub revision: String,
    pub capacity_text: String,
    pub segment: String,
    pub archived: String,
    pub threshold_text: String,
    pub notes: String,
    pub quota_units_text: String,
    pub status_label: String,
}

pub fn to_tiered_egress_cycle_view(item: &TieredEgressCycle) -> TieredEgressCycleView {
    TieredEgressCycleView {
        code_text: text::truncate(&item.code, 40),
        name: text::truncate(&item.name, 40),
        minimum: format!("{:.2}", item.minimum),
        revision: item.revision.to_string(),
        capacity_text: item.capacity.to_string(),
        segment: text::truncate(&item.segment, 40),
        archived: format!("{}", item.archived),
        threshold_text: format!("{:.2}", item.threshold),
        notes: text::truncate(&item.notes, 40),
        quota_units_text: item.quota_units.to_string(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_tiered_egress_cycle_views(items: &[TieredEgressCycle]) -> Vec<TieredEgressCycleView> {
    let mut views: Vec<TieredEgressCycleView> = Vec::new();
    for item in items.iter() {
        views.push(to_tiered_egress_cycle_view(item));
    }
    views
}

pub fn sum_tiered_egress_cycle_view(view: &TieredEgressCycleView) -> Result<f64, String> {
    let parsed_minimum = view
        .minimum
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse minimum: {err}"))?;
    let parsed_revision = view
        .revision
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse revision: {err}"))?;
    let parsed_capacity = view
        .capacity_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse capacity: {err}"))?;
    Ok(parsed_minimum + (parsed_revision as f64) + (parsed_capacity as f64))
}
