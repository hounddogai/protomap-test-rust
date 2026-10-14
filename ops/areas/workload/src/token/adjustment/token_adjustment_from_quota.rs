use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::variance as trust_domain_variance;

pub fn summarize_scheduled_trust_domain_quotas_for_adjustment(
    items: &[trust_domain_variance::ScheduledTrustDomainQuota],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_exposure();
    }
    totals
}

pub fn derive_token_adjustments_from_quota(
    items: &[trust_domain_variance::ScheduledTrustDomainQuota],
) -> Vec<TokenAdjustment> {
    let mut out: Vec<TokenAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_token_adjustment(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn classify_token_adjustment(item: &TokenAdjustment) -> String {
    trust_domain_variance::classify_scheduled_trust_domain_quota(item.projected_exposure())
}
