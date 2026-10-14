use super::*;

use std::collections::BTreeMap;

use crate::migration::proposal as migration_proposal;

pub fn summarize_legacy_migration_checkpoints_for_draft(
    items: &[migration_proposal::LegacyMigrationCheckpoint],
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

pub fn derive_seasonal_migration_drafts_from_checkpoint(
    items: &[migration_proposal::LegacyMigrationCheckpoint],
) -> Vec<SeasonalMigrationDraft> {
    let mut out: Vec<SeasonalMigrationDraft> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_migration_draft(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn settle_seasonal_migration_draft(item: &SeasonalMigrationDraft) -> String {
    migration_proposal::manual_migration_estimate_bucket(item.adjusted_exposure())
}
