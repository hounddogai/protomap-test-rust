use super::*;

use std::collections::BTreeMap;

use crate::limit::digest as limit_digest;

pub fn summarize_limit_scores_for_cycle(items: &[limit_digest::LimitScore]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_exposure();
    }
    totals
}

pub fn derive_scheduled_limit_cycles_from_score(items: &[limit_digest::LimitScore]) -> Vec<ScheduledLimitCycle> {
    let mut out: Vec<ScheduledLimitCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_limit_cycle(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn plan_scheduled_limit_cycle(item: &ScheduledLimitCycle) -> String {
    limit_digest::limit_score_bucket(item.effective_volume())
}
