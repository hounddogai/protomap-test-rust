use super::*;

use std::collections::BTreeMap;

use crate::silence::slot as silence_slot;

pub fn summarize_legacy_silence_budgets_for_group(
    items: &[silence_slot::LegacySilenceBudget],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_legacy_probe_groups_from_budget(items: &[silence_slot::LegacySilenceBudget]) -> Vec<LegacyProbeGroup> {
    let mut out: Vec<LegacyProbeGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_legacy_probe_group(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_legacy_probe_group(item: &LegacyProbeGroup) -> String {
    silence_slot::normalize_manual_silence_slot_code(&item.code)
}
