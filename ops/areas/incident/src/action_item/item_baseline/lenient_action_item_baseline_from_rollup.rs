use super::*;

use std::collections::BTreeMap;

use crate::timeline::policy as timeline_policy;

pub fn summarize_lenient_timeline_rollups_for_baseline(
    items: &[timeline_policy::LenientTimelineRollup],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_lenient_action_item_baselines_from_rollup(
    items: &[timeline_policy::LenientTimelineRollup],
) -> Vec<LenientActionItemBaseline> {
    let mut out: Vec<LenientActionItemBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_action_item_baseline(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn score_lenient_action_item_baseline(item: &LenientActionItemBaseline) -> String {
    timeline_policy::normalize_timeline_profile_code(&item.code)
}
