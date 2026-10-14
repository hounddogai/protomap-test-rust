use super::*;

use std::collections::BTreeMap;

use crate::migration::reservation as migration_reservation;

pub fn summarize_migration_mappings_for_draft(
    items: &[migration_reservation::MigrationMapping],
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

pub fn derive_tiered_plan_drafts_from_mapping(
    items: &[migration_reservation::MigrationMapping],
) -> Vec<TieredPlanDraft> {
    let mut out: Vec<TieredPlanDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_plan_draft(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_tiered_plan_draft(item: &TieredPlanDraft) -> String {
    migration_reservation::normalize_migration_ledger_code(&item.code)
}
