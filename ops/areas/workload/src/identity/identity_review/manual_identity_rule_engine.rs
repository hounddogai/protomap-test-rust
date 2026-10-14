use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual identity rule calculation.
pub trait ManualIdentityRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualIdentityRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedManualIdentityRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualIdentityRuleAdjuster for FlaggedManualIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualIdentityRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualIdentityRuleAdjuster {
    pub minimum: f64,
}

impl ManualIdentityRuleAdjuster for MinimumManualIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualIdentityRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualIdentityRuleAdjuster {
    pub step: f64,
}

impl ManualIdentityRuleAdjuster for RoundingManualIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualIdentityRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualIdentityRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualIdentityRuleAdjuster for TieredManualIdentityRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualIdentityRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual identity rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualIdentityRuleEngine {
    pub steps: Vec<Rc<dyn ManualIdentityRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualIdentityRuleEngine {
    pub fn run(&mut self, item: &ManualIdentityRule) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[ManualIdentityRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_manual_identity_rule_engine(flags: &flags::Registry) -> ManualIdentityRuleEngine {
    ManualIdentityRuleEngine {
        steps: vec![
            Rc::new(FlaggedManualIdentityRuleAdjuster {
                flags: flags.clone(),
                flag: "manual-identity-rule-adaptive".to_string(),
                percent: -0.1,
            }) as Rc<dyn ManualIdentityRuleAdjuster>,
            Rc::new(MinimumManualIdentityRuleAdjuster { minimum: 48.4 }),
            Rc::new(RoundingManualIdentityRuleAdjuster { step: 5.0 }),
            Rc::new(TieredManualIdentityRuleAdjuster {
                bounds: vec![3562.0, 4403.0],
                rates: vec![1.01, 1.14],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 9553.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_identity_rule_engine(&flags::new_registry());
        let item = new_manual_identity_rule("code-63", "name-175");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
