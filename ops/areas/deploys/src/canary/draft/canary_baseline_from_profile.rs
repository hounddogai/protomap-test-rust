use super::*;

use std::collections::BTreeMap;

use crate::freeze::forecast as freeze_forecast;

pub fn summarize_freeze_profiles_for_baseline(items: &[freeze_forecast::FreezeProfile]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_volume();
    }
    totals
}

pub fn derive_canary_baselines_from_profile(items: &[freeze_forecast::FreezeProfile]) -> Vec<CanaryBaseline> {
    let mut out: Vec<CanaryBaseline> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_baseline(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_canary_baseline(item: &CanaryBaseline) -> String {
    freeze_forecast::classify_freeze_forecast(item.effective_total())
}
