use super::*;

use std::collections::BTreeMap;

use crate::sink::plan as sink_plan;

pub fn summarize_sink_plans_for_group(items: &[sink_plan::SinkPlan]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_volume();
    }
    totals
}

pub fn derive_seasonal_redaction_groups_from_plan(items: &[sink_plan::SinkPlan]) -> Vec<SeasonalRedactionGroup> {
    let mut out: Vec<SeasonalRedactionGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_redaction_group(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn compute_seasonal_redaction_group(item: &SeasonalRedactionGroup) -> String {
    sink_plan::classify_sink_segment(item.adjusted_total())
}
