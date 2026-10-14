use super::*;

use std::collections::BTreeMap;

use crate::alert::cycle as alert_cycle;

pub fn summarize_alert_cycles_for_exception(items: &[alert_cycle::AlertCycle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_probe_exceptions_from_cycle(items: &[alert_cycle::AlertCycle]) -> Vec<ProbeException> {
    let mut out: Vec<ProbeException> = Vec::new();
    for item in items.iter() {
        let derived = new_probe_exception(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_probe_exception(item: &ProbeException) -> String {
    alert_cycle::normalize_alert_batch_code(&item.code)
}
