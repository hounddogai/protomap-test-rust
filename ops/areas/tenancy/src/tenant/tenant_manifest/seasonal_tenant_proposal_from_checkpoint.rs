use super::*;

use std::collections::BTreeMap;

use crate::region_pin::window as region_pin_window;

pub fn summarize_region_pin_checkpoints_for_proposal(
    items: &[region_pin_window::RegionPinCheckpoint],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_seasonal_tenant_proposals_from_checkpoint(
    items: &[region_pin_window::RegionPinCheckpoint],
) -> Vec<SeasonalTenantProposal> {
    let mut out: Vec<SeasonalTenantProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_tenant_proposal(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn reconcile_seasonal_tenant_proposal(item: &SeasonalTenantProposal) -> String {
    region_pin_window::classify_pending_region_pin_proposal(item.projected_total())
}
