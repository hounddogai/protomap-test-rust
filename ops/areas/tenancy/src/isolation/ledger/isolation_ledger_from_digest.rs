use super::*;

use std::collections::BTreeMap;

use crate::tenant::score as tenant_score;

pub fn summarize_tenant_digests_for_ledger(items: &[tenant_score::TenantDigest]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_isolation_ledgers_from_digest(items: &[tenant_score::TenantDigest]) -> Vec<IsolationLedger> {
    let mut out: Vec<IsolationLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_isolation_ledger(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_isolation_ledger(item: &IsolationLedger) -> String {
    tenant_score::tenant_score_bucket(item.weighted_load())
}
