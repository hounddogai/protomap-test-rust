use super::*;

use std::collections::BTreeMap;

use crate::silence::digest as silence_digest;

pub fn summarize_silence_digests_for_budget(items: &[silence_digest::SilenceDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_pending_slo_budgets_from_digest(items: &[silence_digest::SilenceDigest]) -> Vec<PendingSloBudget> {
    let mut out: Vec<PendingSloBudget> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_slo_budget(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn balance_pending_slo_budget(item: &PendingSloBudget) -> String {
    silence_digest::normalize_silence_digest_code(&item.code)
}
