use super::*;

use std::collections::BTreeMap;

use crate::peering::peering_plan as peering_peering_plan;

pub fn summarize_rolling_peering_tiers_for_draft(
    items: &[peering_peering_plan::RollingPeeringTier],
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

pub fn derive_lenient_peering_drafts_from_tier(
    items: &[peering_peering_plan::RollingPeeringTier],
) -> Vec<LenientPeeringDraft> {
    let mut out: Vec<LenientPeeringDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_peering_draft(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn estimate_lenient_peering_draft(item: &LenientPeeringDraft) -> String {
    peering_peering_plan::classify_scheduled_peering_metric(item.effective_exposure())
}
