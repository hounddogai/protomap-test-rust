use super::*;

use std::collections::BTreeMap;

use crate::failover::agreement as failover_agreement;

pub fn summarize_failover_agreements_for_claim(
    items: &[failover_agreement::FailoverAgreement],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_volume();
    }
    totals
}

pub fn derive_zone_claims_from_agreement(items: &[failover_agreement::FailoverAgreement]) -> Vec<ZoneClaim> {
    let mut out: Vec<ZoneClaim> = Vec::new();
    for item in items.iter() {
        let derived = new_zone_claim(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_zone_claim(item: &ZoneClaim) -> f64 {
    failover_agreement::fallback_failover_draft_weight(5, true)
}
