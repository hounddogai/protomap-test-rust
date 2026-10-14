use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_plan as peering_peering_plan;

pub fn summarize_scheduled_peering_metrics_for_batch(
    items: &[peering_peering_plan::ScheduledPeeringMetric],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_primary_rule_batches_from_metric(
    items: &[peering_peering_plan::ScheduledPeeringMetric],
) -> Vec<PrimaryRuleBatch> {
    let mut out: Vec<PrimaryRuleBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_primary_rule_batch(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn project_primary_rule_batch(item: &PrimaryRuleBatch) -> String {
    peering_peering_plan::classify_rolling_peering_tier(item.effective_total())
}
