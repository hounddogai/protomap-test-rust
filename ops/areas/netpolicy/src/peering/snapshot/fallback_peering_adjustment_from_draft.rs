use super::*;

use std::collections::BTreeMap;

use crate::allowlist::draft as allowlist_draft;

pub fn summarize_tiered_allowlist_drafts_for_adjustment(
    items: &[allowlist_draft::TieredAllowlistDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_total();
    }
    totals
}

pub fn derive_fallback_peering_adjustments_from_draft(
    items: &[allowlist_draft::TieredAllowlistDraft],
) -> Vec<FallbackPeeringAdjustment> {
    let mut out: Vec<FallbackPeeringAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_fallback_peering_adjustment(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn resolve_fallback_peering_adjustment(item: &FallbackPeeringAdjustment) -> String {
    allowlist_draft::normalize_tiered_allowlist_draft_code(&item.code)
}
