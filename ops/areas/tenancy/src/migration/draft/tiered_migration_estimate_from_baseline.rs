use super::*;

use std::collections::BTreeMap;

use crate::suspension::agreement as suspension_agreement;

pub fn summarize_adaptive_suspension_baselines_for_estimate(
    items: &[suspension_agreement::AdaptiveSuspensionBaseline],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_tiered_migration_estimates_from_baseline(
    items: &[suspension_agreement::AdaptiveSuspensionBaseline],
) -> Vec<TieredMigrationEstimate> {
    let mut out: Vec<TieredMigrationEstimate> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_migration_estimate(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn derive_tiered_migration_estimate(item: &TieredMigrationEstimate) -> String {
    suspension_agreement::adaptive_suspension_checkpoint_bucket(item.measured_value())
}
