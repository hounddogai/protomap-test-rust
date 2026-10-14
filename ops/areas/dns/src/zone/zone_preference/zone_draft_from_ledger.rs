use super::*;

use std::collections::BTreeMap;

use crate::health_check::preference as health_check_preference;

pub fn summarize_batched_health_check_ledgers_for_draft(
    items: &[health_check_preference::BatchedHealthCheckLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_zone_drafts_from_ledger(items: &[health_check_preference::BatchedHealthCheckLedger]) -> Vec<ZoneDraft> {
    let mut out: Vec<ZoneDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_zone_draft(item: &ZoneDraft) -> String {
    health_check_preference::classify_batched_health_check_ledger(item.weighted_value())
}
