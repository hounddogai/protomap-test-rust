use super::*;

use std::collections::BTreeMap;

use crate::page::override_item as page_override_item;

pub fn summarize_page_overrides_for_manifest(items: &[page_override_item::PageOverride]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_value();
    }
    totals
}

pub fn derive_escalation_manifests_from_override(
    items: &[page_override_item::PageOverride],
) -> Vec<EscalationManifest> {
    let mut out: Vec<EscalationManifest> = Vec::new();
    for item in items.iter() {
        let derived = new_escalation_manifest(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn resolve_escalation_manifest(item: &EscalationManifest) -> String {
    page_override_item::classify_page_override(item.adjusted_volume())
}
