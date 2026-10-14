use super::*;

use std::collections::BTreeMap;

use crate::alert::proposal as alert_proposal;

pub fn summarize_strict_alert_bundles_for_cycle(items: &[alert_proposal::StrictAlertBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_global_alert_cycles_from_bundle(items: &[alert_proposal::StrictAlertBundle]) -> Vec<GlobalAlertCycle> {
    let mut out: Vec<GlobalAlertCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_global_alert_cycle(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_global_alert_cycle(item: &GlobalAlertCycle) -> String {
    alert_proposal::normalize_rolling_alert_proposal_code(&item.code)
}
