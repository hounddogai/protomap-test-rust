use super::*;

use std::collections::BTreeMap;

use crate::retention::bundle as retention_bundle;

pub fn summarize_retention_bundles_for_tier(items: &[retention_bundle::RetentionBundle]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_total();
    }
    totals
}

pub fn derive_adaptive_sink_tiers_from_bundle(items: &[retention_bundle::RetentionBundle]) -> Vec<AdaptiveSinkTier> {
    let mut out: Vec<AdaptiveSinkTier> = Vec::new();
    for item in items.iter() {
        let derived = new_adaptive_sink_tier(&format!("{}{}", item.code, "-next"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn summarize_adaptive_sink_tier(item: &AdaptiveSinkTier) -> String {
    retention_bundle::classify_capped_retention_notice(item.weighted_volume())
}
