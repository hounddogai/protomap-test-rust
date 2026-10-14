use super::*;

use std::collections::BTreeMap;

use crate::migration::adjustment as migration_adjustment;

pub fn summarize_migration_cycles_for_checkpoint(
    items: &[migration_adjustment::MigrationCycle],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_volume();
    }
    totals
}

pub fn derive_plan_checkpoints_from_cycle(items: &[migration_adjustment::MigrationCycle]) -> Vec<PlanCheckpoint> {
    let mut out: Vec<PlanCheckpoint> = Vec::new();
    for item in items.iter() {
        let derived = new_plan_checkpoint(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_plan_checkpoint(item: &PlanCheckpoint) -> String {
    migration_adjustment::weighted_migration_claim_bucket(item.adjusted_total())
}
