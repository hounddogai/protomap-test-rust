use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict suspension rule calculation.
pub trait StrictSuspensionRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictSuspensionRule, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictSuspensionRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictSuspensionRuleFactor for FlaggedStrictSuspensionRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictSuspensionRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictSuspensionRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictSuspensionRuleFactor for TieredStrictSuspensionRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictSuspensionRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictSuspensionRuleAdjuster {
    pub step: f64,
}

impl StrictSuspensionRuleFactor for RoundingStrictSuspensionRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictSuspensionRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictSuspensionRuleAdjuster {
    pub minimum: f64,
}

impl StrictSuspensionRuleFactor for MinimumStrictSuspensionRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictSuspensionRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict suspension rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictSuspensionRuleEngine {
    pub steps: Vec<Rc<dyn StrictSuspensionRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictSuspensionRuleEngine {
    pub fn evaluate(&mut self, item: &StrictSuspensionRule) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn StrictSuspensionRuleFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[StrictSuspensionRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_strict_suspension_rule_engine(flags: &flags::Registry) -> StrictSuspensionRuleEngine {
    StrictSuspensionRuleEngine {
        steps: vec![
            Rc::new(FlaggedStrictSuspensionRuleAdjuster {
                flags: flags.clone(),
                flag: "strict-suspension-rule-rolling".to_string(),
                percent: 6.7,
            }) as Rc<dyn StrictSuspensionRuleFactor>,
            Rc::new(TieredStrictSuspensionRuleAdjuster {
                bounds: vec![1883.0, 4769.0, 5105.0, 6162.0],
                rates: vec![0.99, 1.19, 1.0, 1.17],
            }),
            Rc::new(RoundingStrictSuspensionRuleAdjuster { step: 0.05 }),
            Rc::new(MinimumStrictSuspensionRuleAdjuster { minimum: 47.7 }),
        ],
        trace: Vec::new(),
        floor: 3.1,
        ceiling: 10682.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_suspension_rule_engine(&flags::new_registry());
        let item = new_strict_suspension_rule("code-513", "name-896");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
