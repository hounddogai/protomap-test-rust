use super::*;

use std::collections::BTreeMap;

use crate::retention::record_item as retention_record_item;

pub fn summarize_tiered_retention_entries_for_adjustment(
    items: &[retention_record_item::TieredRetentionEntry],
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

pub fn derive_retention_adjustments_from_entry(
    items: &[retention_record_item::TieredRetentionEntry],
) -> Vec<RetentionAdjustment> {
    let mut out: Vec<RetentionAdjustment> = Vec::new();
    for item in items.iter() {
        let derived = new_retention_adjustment(&format!("{}{}", item.code, "-copy"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn allocate_retention_adjustment(item: &RetentionAdjustment) -> String {
    retention_record_item::retention_record_bucket(item.projected_volume())
}
