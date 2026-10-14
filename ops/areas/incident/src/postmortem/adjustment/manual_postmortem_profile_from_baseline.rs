use super::*;

use std::collections::BTreeMap;

use crate::action_item::item_baseline as action_item_item_baseline;

pub fn summarize_lenient_action_item_baselines_for_profile(
    items: &[action_item_item_baseline::LenientActionItemBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_manual_postmortem_profiles_from_baseline(
    items: &[action_item_item_baseline::LenientActionItemBaseline],
) -> Vec<ManualPostmortemProfile> {
    let mut out: Vec<ManualPostmortemProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_manual_postmortem_profile(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_manual_postmortem_profile(item: &ManualPostmortemProfile) -> f64 {
    action_item_item_baseline::seasonal_action_item_batch_weight(2, true)
}
