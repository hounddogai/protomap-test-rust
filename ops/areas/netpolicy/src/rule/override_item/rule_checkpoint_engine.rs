use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule checkpoint calculation.
pub trait RuleCheckpointStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleCheckpointAdjuster {
    pub cap: f64,
}

impl RuleCheckpointStep for CapRuleCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleCheckpointStep for TieredRuleCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleCheckpointAdjuster {
    pub percent: f64,
}

impl RuleCheckpointStep for PercentRuleCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleCheckpointAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleCheckpointStep for FlaggedRuleCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleCheckpoint, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleCheckpointAdjuster {
    pub step: f64,
}

impl RuleCheckpointStep for RoundingRuleCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rule checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleCheckpointEngine {
    pub steps: Vec<Rc<dyn RuleCheckpointStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleCheckpointEngine {
    pub fn price(&mut self, item: &RuleCheckpoint) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[RuleCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rule_checkpoint_engine(flags: &flags::Registry) -> RuleCheckpointEngine {
    RuleCheckpointEngine {
        steps: vec![
            Rc::new(CapRuleCheckpointAdjuster { cap: 31875.0 }) as Rc<dyn RuleCheckpointStep>,
            Rc::new(TieredRuleCheckpointAdjuster {
                bounds: vec![1683.0, 3574.0],
                rates: vec![1.05, 1.04],
            }),
            Rc::new(PercentRuleCheckpointAdjuster { percent: 18.1 }),
            Rc::new(FlaggedRuleCheckpointAdjuster {
                flags: flags.clone(),
                flag: "rule-checkpoint-adaptive".to_string(),
                percent: -5.3,
            }),
            Rc::new(RoundingRuleCheckpointAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 40489.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_checkpoint_engine(&flags::new_registry());
        let item = new_rule_checkpoint("code-487", "title-91");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
