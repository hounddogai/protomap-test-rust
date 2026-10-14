use super::*;

use std::collections::BTreeMap;

use crate::timeline::forecast as timeline_forecast;

pub fn summarize_primary_timeline_rules_for_policy(
    items: &[timeline_forecast::PrimaryTimelineRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_seasonal_action_item_policies_from_rule(
    items: &[timeline_forecast::PrimaryTimelineRule],
) -> Vec<SeasonalActionItemPolicy> {
    let mut out: Vec<SeasonalActionItemPolicy> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_action_item_policy(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_seasonal_action_item_policy(item: &SeasonalActionItemPolicy) -> String {
    timeline_forecast::primary_timeline_rule_bucket(item.projected_load())
}
