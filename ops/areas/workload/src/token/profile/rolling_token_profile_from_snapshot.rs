use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::snapshot as trust_domain_snapshot;

pub fn summarize_trust_domain_snapshots_for_profile(
    items: &[trust_domain_snapshot::TrustDomainSnapshot],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_value();
    }
    totals
}

pub fn derive_rolling_token_profiles_from_snapshot(
    items: &[trust_domain_snapshot::TrustDomainSnapshot],
) -> Vec<RollingTokenProfile> {
    let mut out: Vec<RollingTokenProfile> = Vec::new();
    for item in items.iter() {
        let derived = new_rolling_token_profile(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn balance_rolling_token_profile(item: &RollingTokenProfile) -> f64 {
    trust_domain_snapshot::trust_domain_snapshot_weight(6, true)
}
