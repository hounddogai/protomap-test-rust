use super::*;

use std::collections::BTreeMap;

use crate::region_pin::proposal as region_pin_proposal;

pub fn summarize_adaptive_region_pin_ledgers_for_draft(
    items: &[region_pin_proposal::AdaptiveRegionPinLedger],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_total();
    }
    totals
}

pub fn derive_suspension_drafts_from_ledger(
    items: &[region_pin_proposal::AdaptiveRegionPinLedger],
) -> Vec<SuspensionDraft> {
    let mut out: Vec<SuspensionDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_suspension_draft(item: &SuspensionDraft) -> String {
    region_pin_proposal::region_pin_forecast_bucket(item.adjusted_total())
}
