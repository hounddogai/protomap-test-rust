use super::*;

use std::collections::BTreeMap;

use crate::binding::binding_group as binding_binding_group;

pub fn summarize_pending_binding_windows_for_cycle(
    items: &[binding_binding_group::PendingBindingWindow],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_credential_cycles_from_window(
    items: &[binding_binding_group::PendingBindingWindow],
) -> Vec<CredentialCycle> {
    let mut out: Vec<CredentialCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_credential_cycle(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_credential_cycle(item: &CredentialCycle) -> String {
    binding_binding_group::classify_tiered_binding_group(item.net_total())
}
