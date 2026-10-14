use super::*;

use std::collections::BTreeMap;

use crate::release::template as release_template;

pub fn summarize_release_templates_for_ledger(items: &[release_template::ReleaseTemplate]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_rollout_ledgers_from_template(items: &[release_template::ReleaseTemplate]) -> Vec<RolloutLedger> {
    let mut out: Vec<RolloutLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_rollout_ledger(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_rollout_ledger(item: &RolloutLedger) -> String {
    release_template::normalize_release_baseline_code(&item.code)
}
