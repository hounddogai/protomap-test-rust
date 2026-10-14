use super::*;

use std::collections::BTreeMap;

use crate::grant::plan as grant_plan;

pub fn summarize_grant_notices_for_tier(items: &[grant_plan::GrantNotice]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_overage_tiers_from_notice(items: &[grant_plan::GrantNotice]) -> Vec<OverageTier> {
    let mut out: Vec<OverageTier> = Vec::new();
    for item in items.iter() {
        let derived = new_overage_tier(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn split_overage_tier(item: &OverageTier) -> String {
    grant_plan::tiered_grant_ledger_bucket(item.adjusted_total())
}
