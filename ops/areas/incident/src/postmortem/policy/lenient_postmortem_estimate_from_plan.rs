use super::*;

use std::collections::BTreeMap;

use crate::postmortem::draft as postmortem_draft;

pub fn summarize_legacy_postmortem_plans_for_estimate(
    items: &[postmortem_draft::LegacyPostmortemPlan],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_load();
    }
    totals
}

pub fn derive_lenient_postmortem_estimates_from_plan(
    items: &[postmortem_draft::LegacyPostmortemPlan],
) -> Vec<LenientPostmortemEstimate> {
    let mut out: Vec<LenientPostmortemEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_postmortem_estimate(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_lenient_postmortem_estimate(item: &LenientPostmortemEstimate) -> f64 {
    postmortem_draft::legacy_postmortem_plan_weight(5, true)
}
