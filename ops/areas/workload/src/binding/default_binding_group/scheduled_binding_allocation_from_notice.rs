use super::*;

use std::collections::BTreeMap;

use crate::trust_domain::assignment as trust_domain_assignment;

pub fn summarize_regional_trust_domain_notices_for_allocation(
    items: &[trust_domain_assignment::RegionalTrustDomainNotice],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.projected_load();
    }
    totals
}

pub fn derive_scheduled_binding_allocations_from_notice(
    items: &[trust_domain_assignment::RegionalTrustDomainNotice],
) -> Vec<ScheduledBindingAllocation> {
    let mut out: Vec<ScheduledBindingAllocation> = Vec::new();
    for item in items.iter() {
        let derived = new_scheduled_binding_allocation(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn reconcile_scheduled_binding_allocation(item: &ScheduledBindingAllocation) -> String {
    trust_domain_assignment::classify_regional_trust_domain_notice(item.effective_volume())
}
