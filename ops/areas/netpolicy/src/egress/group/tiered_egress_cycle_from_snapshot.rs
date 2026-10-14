use super::*;

use std::collections::BTreeMap;

use crate::flow_log::snapshot as flow_log_snapshot;

pub fn summarize_flow_log_snapshots_for_cycle(items: &[flow_log_snapshot::FlowLogSnapshot]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.measured_value();
    }
    totals
}

pub fn derive_tiered_egress_cycles_from_snapshot(
    items: &[flow_log_snapshot::FlowLogSnapshot],
) -> Vec<TieredEgressCycle> {
    let mut out: Vec<TieredEgressCycle> = Vec::new();
    for item in items.iter() {
        let derived = new_tiered_egress_cycle(&format!("{}{}", item.code, "-plan"), &item.label());
        out.push(derived);
    }
    out
}

pub fn allocate_tiered_egress_cycle(item: &TieredEgressCycle) -> String {
    flow_log_snapshot::normalize_flow_log_snapshot_code(&item.code)
}
