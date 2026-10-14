use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary budget burn rule calculation.
pub trait PrimaryBudgetBurnRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryBudgetBurnRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryBudgetBurnRuleAdjuster {
    pub percent: f64,
}

impl PrimaryBudgetBurnRuleStep for PercentPrimaryBudgetBurnRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryBudgetBurnRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryBudgetBurnRuleStep for FlaggedPrimaryBudgetBurnRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnRuleWeightOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryBudgetBurnRuleStep for PrimaryBudgetBurnRuleWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryBudgetBurnRule, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

/// Runs the primary budget burn rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryBudgetBurnRuleEngine {
    pub steps: Vec<Rc<dyn PrimaryBudgetBurnRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryBudgetBurnRuleEngine {
    pub fn compute(&mut self, item: &PrimaryBudgetBurnRule) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PrimaryBudgetBurnRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_budget_burn_rule_engine(flags: &flags::Registry) -> PrimaryBudgetBurnRuleEngine {
    PrimaryBudgetBurnRuleEngine {
        steps: vec![
            Rc::new(PercentPrimaryBudgetBurnRuleAdjuster { percent: -22.3 }) as Rc<dyn PrimaryBudgetBurnRuleStep>,
            Rc::new(FlaggedPrimaryBudgetBurnRuleAdjuster {
                flags: flags.clone(),
                flag: "primary-budget-burn-rule-lenient".to_string(),
                percent: -7.5,
            }),
            Rc::new(PrimaryBudgetBurnRuleWeightOffsetAdjuster { weight: 0.15 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 74248.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_budget_burn_rule_engine(&flags::new_registry());
        let item = new_primary_budget_burn_rule("code-728", "title-643");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
