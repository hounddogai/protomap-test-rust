use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule window calculation.
pub trait RuleWindowModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleWindowAdjuster {
    pub minimum: f64,
}

impl RuleWindowModifier for MinimumRuleWindowAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleWindow, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleWindowModifier for FlaggedRuleWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleWindowModifier for TieredRuleWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleWindowAdjuster {
    pub step: f64,
}

impl RuleWindowModifier for RoundingRuleWindowAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleWindow, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rule window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleWindowEngine {
    pub steps: Vec<Rc<dyn RuleWindowModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleWindowEngine {
    pub fn compute(&mut self, item: &RuleWindow) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleWindowModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_window_engine(flags: &flags::Registry) -> RuleWindowEngine {
    RuleWindowEngine {
        steps: vec![
            Rc::new(MinimumRuleWindowAdjuster { minimum: 20.8 }) as Rc<dyn RuleWindowModifier>,
            Rc::new(FlaggedRuleWindowAdjuster {
                flags: flags.clone(),
                flag: "rule-window-rolling".to_string(),
                percent: 8.1,
            }),
            Rc::new(TieredRuleWindowAdjuster {
                bounds: vec![1591.0, 8839.0],
                rates: vec![0.94, 0.98],
            }),
            Rc::new(RoundingRuleWindowAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 20159.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_window_engine(&flags::new_registry());
        let item = new_rule_window("code-664", "label-686");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
