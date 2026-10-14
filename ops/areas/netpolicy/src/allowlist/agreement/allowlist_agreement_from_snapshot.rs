use super::*;

use std::collections::BTreeMap;

use crate::allowlist::allocation as allowlist_allocation;

pub fn summarize_strict_allowlist_snapshots_for_agreement(
    items: &[allowlist_allocation::StrictAllowlistSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_load();
    }
    totals
}

pub fn derive_allowlist_agreements_from_snapshot(
    items: &[allowlist_allocation::StrictAllowlistSnapshot],
) -> Vec<AllowlistAgreement> {
    let mut out: Vec<AllowlistAgreement> = Vec::new();
    for item in items.iter() {
        let derived = new_allowlist_agreement(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn project_allowlist_agreement(item: &AllowlistAgreement) -> String {
    allowlist_allocation::normalize_legacy_allowlist_adjustment_code(&item.code)
}
