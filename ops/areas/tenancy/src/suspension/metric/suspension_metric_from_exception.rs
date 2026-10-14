use super::*;

use std::collections::BTreeMap;

use crate::isolation::ledger as isolation_ledger;

pub fn summarize_isolation_exceptions_for_metric(
    items: &[isolation_ledger::IsolationException],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_suspension_metrics_from_exception(
    items: &[isolation_ledger::IsolationException],
) -> Vec<SuspensionMetric> {
    let mut out: Vec<SuspensionMetric> = Vec::new();
    for item in items.iter() {
        let derived = new_suspension_metric(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_suspension_metric(item: &SuspensionMetric) -> String {
    isolation_ledger::classify_isolation_milestone(item.adjusted_value())
}
