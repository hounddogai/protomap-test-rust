use super::*;

use std::collections::BTreeMap;

use crate::egress::entry as egress_entry;

pub fn summarize_rolling_egress_entries_for_template(
    items: &[egress_entry::RollingEgressEntry],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_allowlist_templates_from_entry(items: &[egress_entry::RollingEgressEntry]) -> Vec<AllowlistTemplate> {
    let mut out: Vec<AllowlistTemplate> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_template(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn merge_allowlist_template(item: &AllowlistTemplate) -> String {
    egress_entry::classify_seasonal_egress_batch(item.effective_volume())
}
