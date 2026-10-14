use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::checkpoint as trust_domain_checkpoint;

pub fn summarize_trust_domain_targets_for_slot(
    items: &[trust_domain_checkpoint::TrustDomainTarget],
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

pub fn derive_token_slots_from_target(items: &[trust_domain_checkpoint::TrustDomainTarget]) -> Vec<TokenSlot> {
    let mut out: Vec<TokenSlot> = Vec::new();
    for item in items.iter() {
        let derived = new_token_slot(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_token_slot(item: &TokenSlot) -> String {
    trust_domain_checkpoint::trust_domain_record_bucket(item.effective_exposure())
}
