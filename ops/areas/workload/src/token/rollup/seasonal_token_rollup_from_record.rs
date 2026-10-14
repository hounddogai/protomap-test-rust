use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::cycle as trust_domain_cycle;

pub fn summarize_adaptive_trust_domain_records_for_rollup(
    items: &[trust_domain_cycle::AdaptiveTrustDomainRecord],
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

pub fn derive_seasonal_token_rollups_from_record(
    items: &[trust_domain_cycle::AdaptiveTrustDomainRecord],
) -> Vec<SeasonalTokenRollup> {
    let mut out: Vec<SeasonalTokenRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_token_rollup(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn split_seasonal_token_rollup(item: &SeasonalTokenRollup) -> String {
    trust_domain_cycle::legacy_trust_domain_reservation_bucket(item.net_volume())
}
