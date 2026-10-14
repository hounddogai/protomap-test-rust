use super::*;

use std::collections::BTreeMap;

use crate::sink::tier as sink_tier;

pub fn summarize_sink_tiers_for_rollup(items: &[sink_tier::SinkTier]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_redaction_rollups_from_tier(items: &[sink_tier::SinkTier]) -> Vec<RedactionRollup> {
    let mut out: Vec<RedactionRollup> = Vec::new();
    for item in items.iter() {
        let derived = new_redaction_rollup(&format!("{}{}", item.code, "-next"), &item.label());
        out.push(derived);
    }
    out
}

pub fn score_redaction_rollup(item: &RedactionRollup) -> String {
    sink_tier::classify_capped_sink_variance(item.net_load())
}
