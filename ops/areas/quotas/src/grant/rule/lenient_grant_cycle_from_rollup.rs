use super::*;

use std::collections::BTreeMap;

use crate::limit::plan as limit_plan;

pub fn summarize_limit_rollups_for_cycle(items: &[limit_plan::LimitRollup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_lenient_grant_cycles_from_rollup(items: &[limit_plan::LimitRollup]) -> Vec<LenientGrantCycle> {
    let mut out: Vec<LenientGrantCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_grant_cycle(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_lenient_grant_cycle(item: &LenientGrantCycle) -> String {
    limit_plan::normalize_manual_limit_reservation_code(&item.code)
}
