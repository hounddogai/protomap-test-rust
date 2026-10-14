use super::*;

use std::collections::BTreeMap;

use crate::postmortem::postmortem_target as postmortem_postmortem_target;

pub fn summarize_adaptive_postmortem_adjustments_for_batch(
    items: &[postmortem_postmortem_target::AdaptivePostmortemAdjustment],
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

pub fn derive_seasonal_action_item_batches_from_adjustment(
    items: &[postmortem_postmortem_target::AdaptivePostmortemAdjustment],
) -> Vec<SeasonalActionItemBatch> {
    let mut out: Vec<SeasonalActionItemBatch> = Vec::new();
    for item in items.iter() {
        let derived = new_seasonal_action_item_batch(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn compute_seasonal_action_item_batch(item: &SeasonalActionItemBatch) -> String {
    postmortem_postmortem_target::normalize_postmortem_checkpoint_code(&item.code)
}
