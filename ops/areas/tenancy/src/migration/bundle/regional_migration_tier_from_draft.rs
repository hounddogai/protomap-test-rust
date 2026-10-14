use super::*;

use std::collections::BTreeMap;

use crate::suspension::record_item as suspension_record_item;

pub fn summarize_suspension_drafts_for_tier(
    items: &[suspension_record_item::SuspensionDraft],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.adjusted_total();
    }
    totals
}

pub fn derive_regional_migration_tiers_from_draft(
    items: &[suspension_record_item::SuspensionDraft],
) -> Vec<RegionalMigrationTier> {
    let mut out: Vec<RegionalMigrationTier> = Vec::new();
    for item in items.iter() {
        let derived = new_regional_migration_tier(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_regional_migration_tier(item: &RegionalMigrationTier) -> f64 {
    suspension_record_item::capped_suspension_entry_weight(9, true)
}
