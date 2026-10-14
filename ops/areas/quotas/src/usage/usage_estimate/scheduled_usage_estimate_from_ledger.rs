use super::*;

use std::collections::BTreeMap;

use crate::overage::ledger as overage_ledger;

pub fn summarize_overage_ledgers_for_estimate(items: &[overage_ledger::OverageLedger]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_scheduled_usage_estimates_from_ledger(
    items: &[overage_ledger::OverageLedger],
) -> Vec<ScheduledUsageEstimate> {
    let mut out: Vec<ScheduledUsageEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_usage_estimate(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn match_scheduled_usage_estimate(item: &ScheduledUsageEstimate) -> String {
    overage_ledger::normalize_overage_ledger_code(&item.code)
}
