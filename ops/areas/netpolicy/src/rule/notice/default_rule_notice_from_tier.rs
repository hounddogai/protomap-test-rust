use super::*;

use std::collections::BTreeMap;

use crate::peering::record_item as peering_record_item;

pub fn summarize_global_peering_tiers_for_notice(
    items: &[peering_record_item::GlobalPeeringTier],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_default_rule_notices_from_tier(
    items: &[peering_record_item::GlobalPeeringTier],
) -> Vec<DefaultRuleNotice> {
    let mut out: Vec<DefaultRuleNotice> = Vec::new();
    for item in items.iter() {
        let derived = new_default_rule_notice(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn settle_default_rule_notice(item: &DefaultRuleNotice) -> String {
    peering_record_item::normalize_capped_peering_record_code(&item.code)
}
