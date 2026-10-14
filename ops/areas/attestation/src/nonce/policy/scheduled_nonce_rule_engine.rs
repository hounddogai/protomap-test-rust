use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled nonce rule calculation.
pub trait ScheduledNonceRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledNonceRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledNonceRuleAdjuster {
    pub minimum: f64,
}

impl ScheduledNonceRuleModifier for MinimumScheduledNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledNonceRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledNonceRuleScoreOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledNonceRuleModifier for ScheduledNonceRuleScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledNonceRule, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledNonceRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledNonceRuleModifier for FlaggedScheduledNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledNonceRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledNonceRuleAdjuster {
    pub step: f64,
}

impl ScheduledNonceRuleModifier for RoundingScheduledNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledNonceRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledNonceRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledNonceRuleModifier for TieredScheduledNonceRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledNonceRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the scheduled nonce rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledNonceRuleEngine {
    pub steps: Vec<Rc<dyn ScheduledNonceRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledNonceRuleEngine {
    pub fn compute(&mut self, item: &ScheduledNonceRule) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledNonceRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_nonce_rule_engine(flags: &flags::Registry) -> ScheduledNonceRuleEngine {
    ScheduledNonceRuleEngine {
        steps: vec![
            Rc::new(MinimumScheduledNonceRuleAdjuster { minimum: 48.4 }) as Rc<dyn ScheduledNonceRuleModifier>,
            Rc::new(ScheduledNonceRuleScoreOffsetAdjuster { weight: 0.14 }),
            Rc::new(FlaggedScheduledNonceRuleAdjuster {
                flags: flags.clone(),
                flag: "scheduled-nonce-rule-scheduled".to_string(),
                percent: 2.4,
            }),
            Rc::new(RoundingScheduledNonceRuleAdjuster { step: 0.25 }),
            Rc::new(TieredScheduledNonceRuleAdjuster {
                bounds: vec![1892.0, 2992.0, 4471.0, 6882.0],
                rates: vec![1.02, 1.01, 1.09, 1.12],
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 49197.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_nonce_rule_engine(&flags::new_registry());
        let item = new_scheduled_nonce_rule("code-425", "title-230");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
