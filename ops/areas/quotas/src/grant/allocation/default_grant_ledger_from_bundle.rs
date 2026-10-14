use super::*;

use std::collections::BTreeMap;

use crate::overage::bundle as overage_bundle;

pub fn summarize_seasonal_overage_bundles_for_ledger(
    items: &[overage_bundle::SeasonalOverageBundle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_exposure();
    }
    totals
}

pub fn derive_default_grant_ledgers_from_bundle(
    items: &[overage_bundle::SeasonalOverageBundle],
) -> Vec<DefaultGrantLedger> {
    let mut out: Vec<DefaultGrantLedger> = Vec::new();
    for item in items.iter() {
        let derived = new_default_grant_ledger(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn classify_default_grant_ledger(item: &DefaultGrantLedger) -> f64 {
    overage_bundle::seasonal_overage_bundle_weight(4, true)
}
