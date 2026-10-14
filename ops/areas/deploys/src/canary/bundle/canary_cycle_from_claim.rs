use super::*;

use std::collections::BTreeMap;

use crate::rollback::snapshot as rollback_snapshot;

pub fn summarize_seasonal_rollback_claims_for_cycle(
    items: &[rollback_snapshot::SeasonalRollbackClaim],
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

pub fn derive_canary_cycles_from_claim(items: &[rollback_snapshot::SeasonalRollbackClaim]) -> Vec<CanaryCycle> {
    let mut out: Vec<CanaryCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_canary_cycle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_canary_cycle(item: &CanaryCycle) -> String {
    rollback_snapshot::seasonal_rollback_claim_bucket(item.effective_exposure())
}
