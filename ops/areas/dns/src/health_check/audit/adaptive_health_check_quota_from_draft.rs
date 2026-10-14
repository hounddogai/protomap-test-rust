use super::*;

use std::collections::BTreeMap;

use crate::delegation::delegation_draft as delegation_delegation_draft;

pub fn summarize_capped_delegation_drafts_for_quota(
    items: &[delegation_delegation_draft::CappedDelegationDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_adaptive_health_check_quotas_from_draft(
    items: &[delegation_delegation_draft::CappedDelegationDraft],
) -> Vec<AdaptiveHealthCheckQuota> {
    let mut out: Vec<AdaptiveHealthCheckQuota> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_health_check_quota(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_adaptive_health_check_quota(item: &AdaptiveHealthCheckQuota) -> String {
    delegation_delegation_draft::normalize_lenient_delegation_manifest_code(&item.code)
}
