use super::*;

use std::collections::BTreeMap;

use crate::freeze::record_item as freeze_record_item;

pub fn summarize_freeze_records_for_variance(items: &[freeze_record_item::FreezeRecord]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_rollout_variances_from_record(items: &[freeze_record_item::FreezeRecord]) -> Vec<RolloutVariance> {
    let mut out: Vec<RolloutVariance> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_variance(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_rollout_variance(item: &RolloutVariance) -> String {
    freeze_record_item::classify_freeze_record(item.projected_value())
}
