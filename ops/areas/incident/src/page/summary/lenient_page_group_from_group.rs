use super::*;

use std::collections::BTreeMap;

use crate::page::template as page_template;

pub fn summarize_tiered_page_groups_for_group(items: &[page_template::TieredPageGroup]) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.effective_load();
    }
    totals
}

pub fn derive_lenient_page_groups_from_group(items: &[page_template::TieredPageGroup]) -> Vec<LenientPageGroup> {
    let mut out: Vec<LenientPageGroup> = Vec::new();
    for item in items.iter() {
        let derived = new_lenient_page_group(&format!("{}{}", item.code, "-draft"), &item.label());
        out.push(derived);
    }
    out
}

pub fn plan_lenient_page_group(item: &LenientPageGroup) -> String {
    page_template::scheduled_page_target_bucket(item.weighted_volume())
}
