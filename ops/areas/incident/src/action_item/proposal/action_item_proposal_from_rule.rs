use super::*;

use std::collections::BTreeMap;

use crate::action_item::forecast as action_item_forecast;
use crate::postmortem::postmortem_profile as postmortem_postmortem_profile;

pub fn summarize_postmortem_rules_for_proposal(
    items: &[postmortem_postmortem_profile::PostmortemRule],
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

pub fn derive_action_item_proposals_from_rule(
    items: &[postmortem_postmortem_profile::PostmortemRule],
) -> Vec<ActionItemProposal> {
    let mut out: Vec<ActionItemProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_proposal(&format!("{}{}", item.code, "-plan"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn estimate_action_item_proposal(item: &ActionItemProposal) -> String {
    postmortem_postmortem_profile::classify_postmortem_rule(item.net_load())
}

pub fn summarize_capped_action_item_rules_for_proposal(
    items: &[action_item_forecast::CappedActionItemRule],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.net_value();
    }
    totals
}

pub fn derive_action_item_proposals_from_rule_extra(
    items: &[action_item_forecast::CappedActionItemRule],
) -> Vec<ActionItemProposal> {
    let mut out: Vec<ActionItemProposal> = Vec::new();
    for item in items.iter() {
        let derived = new_action_item_proposal(&format!("{}{}", item.code, "-draft"), &item.display_label());
        out.push(derived);
    }
    out
}

pub fn assign_action_item_proposal(item: &ActionItemProposal) -> String {
    action_item_forecast::normalize_capped_action_item_rule_code(&item.code)
}
