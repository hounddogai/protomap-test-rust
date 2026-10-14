use super::*;

use std::collections::BTreeMap;

use crate::alert::rule as alert_rule;

pub fn summarize_alert_rules_for_request(items: &[alert_rule::AlertRule]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_pending_probe_requests_from_rule(items: &[alert_rule::AlertRule]) -> Vec<PendingProbeRequest> {
    let mut out: Vec<PendingProbeRequest> = Vec::new();
    for item in items.iter() {
        let derived = new_pending_probe_request(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn assign_pending_probe_request(item: &PendingProbeRequest) -> String {
    alert_rule::classify_legacy_alert_slot(item.net_exposure())
}
