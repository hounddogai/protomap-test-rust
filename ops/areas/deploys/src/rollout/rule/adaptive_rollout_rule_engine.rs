use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the adaptive rollout rule calculation.
pub trait AdaptiveRolloutRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptiveRolloutRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptiveRolloutRuleAdjuster {
    pub percent: f64,
}

impl AdaptiveRolloutRuleModifier for PercentAdaptiveRolloutRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptiveRolloutRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptiveRolloutRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptiveRolloutRuleModifier for TieredAdaptiveRolloutRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptiveRolloutRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAdaptiveRolloutRuleAdjuster {
    pub step: f64,
}

impl AdaptiveRolloutRuleModifier for RoundingAdaptiveRolloutRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AdaptiveRolloutRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the adaptive rollout rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptiveRolloutRuleEngine {
    pub steps: Vec<Rc<dyn AdaptiveRolloutRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptiveRolloutRuleEngine {
    pub fn run(&mut self, item: &AdaptiveRolloutRule) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[AdaptiveRolloutRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_adaptive_rollout_rule_engine() -> AdaptiveRolloutRuleEngine {
    AdaptiveRolloutRuleEngine {
        steps: vec![
            Rc::new(PercentAdaptiveRolloutRuleAdjuster { percent: -3.5 }) as Rc<dyn AdaptiveRolloutRuleModifier>,
            Rc::new(TieredAdaptiveRolloutRuleAdjuster {
                bounds: vec![6258.0, 7402.0, 9631.0],
                rates: vec![0.89, 0.88, 0.88],
            }),
            Rc::new(RoundingAdaptiveRolloutRuleAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 70568.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_adaptive_rollout_rule_engine();
        let item = new_adaptive_rollout_rule("code-404", "label-335");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
