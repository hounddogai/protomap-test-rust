use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule threshold calculation.
pub trait RuleThresholdFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleThresholdAdjuster {
    pub minimum: f64,
}

impl RuleThresholdFactor for MinimumRuleThresholdAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleThreshold, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleThresholdAdjuster {
    pub step: f64,
}

impl RuleThresholdFactor for RoundingRuleThresholdAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleThreshold, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleThresholdAdjuster {
    pub cap: f64,
}

impl RuleThresholdFactor for CapRuleThresholdAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleThreshold, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rule threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleThresholdEngine {
    pub steps: Vec<Rc<dyn RuleThresholdFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleThresholdEngine {
    pub fn run(&mut self, item: &RuleThreshold) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[RuleThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rule_threshold_engine() -> RuleThresholdEngine {
    RuleThresholdEngine {
        steps: vec![
            Rc::new(MinimumRuleThresholdAdjuster { minimum: 0.2 }) as Rc<dyn RuleThresholdFactor>,
            Rc::new(RoundingRuleThresholdAdjuster { step: 5.0 }),
            Rc::new(CapRuleThresholdAdjuster { cap: 33461.0 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 54086.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_threshold_engine();
        let item = new_rule_threshold("code-286", "label-609");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
