use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary rule group calculation.
pub trait PrimaryRuleGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryRuleGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryRuleGroupAdjuster {
    pub cap: f64,
}

impl PrimaryRuleGroupFactor for CapPrimaryRuleGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryRuleGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryRuleGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryRuleGroupFactor for FlaggedPrimaryRuleGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryRuleGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryRuleGroupAdjuster {
    pub percent: f64,
}

impl PrimaryRuleGroupFactor for PercentPrimaryRuleGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryRuleGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary rule group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryRuleGroupEngine {
    pub steps: Vec<Rc<dyn PrimaryRuleGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryRuleGroupEngine {
    pub fn evaluate(&mut self, item: &PrimaryRuleGroup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryRuleGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_rule_group_engine(flags: &flags::Registry) -> PrimaryRuleGroupEngine {
    PrimaryRuleGroupEngine {
        steps: vec![
            Rc::new(CapPrimaryRuleGroupAdjuster { cap: 19536.0 }) as Rc<dyn PrimaryRuleGroupFactor>,
            Rc::new(FlaggedPrimaryRuleGroupAdjuster {
                flags: flags.clone(),
                flag: "primary-rule-group-scheduled".to_string(),
                percent: -4.4,
            }),
            Rc::new(PercentPrimaryRuleGroupAdjuster { percent: -10.5 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 64610.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_rule_group_engine(&flags::new_registry());
        let item = new_primary_rule_group("code-691", "name-226");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
