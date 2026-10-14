use super::*;

use std::collections::BTreeMap;

use crate::budget_burn::ledger as budget_burn_ledger;

pub fn summarize_legacy_budget_burn_templates_for_rule(
    items: &[budget_burn_ledger::LegacyBudgetBurnTemplate],
) -> BTreeMap<String, f64> {
    let mut totals: BTreeMap<String, f64> = BTreeMap::new();
    for item in items.iter() {
        if item.is_terminal() {
            continue;
        }
        *totals.entry(item.group_key()).or_default() += item.weighted_total();
    }
    totals
}

pub fn derive_budget_burn_rules_from_template(
    items: &[budget_burn_ledger::LegacyBudgetBurnTemplate],
) -> Vec<BudgetBurnRule> {
    let mut out: Vec<BudgetBurnRule> = Vec::new();
    for item in items.iter() {
        let derived = new_budget_burn_rule(&format!("{}{}", item.code, "-copy"), &item.label());
        out.push(derived);
    }
    out
}

pub fn match_budget_burn_rule(item: &BudgetBurnRule) -> String {
    budget_burn_ledger::budget_burn_audit_bucket(item.weighted_load())
}
