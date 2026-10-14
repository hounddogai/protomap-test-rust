use super::*;

use std::collections::BTreeMap;

use crate::allowlist::slot as allowlist_slot;

pub fn summarize_allowlist_notices_for_override(items: &[allowlist_slot::AllowlistNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_value();
    }
    totals
}

pub fn derive_egress_overrides_from_notice(items: &[allowlist_slot::AllowlistNotice]) -> Vec<EgressOverride> {
    let mut out: Vec<EgressOverride> = Vec::new();
    for item in items.iter() {
        let derived = new_egress_override(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}
