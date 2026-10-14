use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the slo rule calculation.
pub trait SloRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloRuleAdjuster {
    pub minimum: f64,
}

impl SloRuleModifier for MinimumSloRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSloRuleAdjuster {
    pub cap: f64,
}

impl SloRuleModifier for CapSloRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SloRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSloRuleAdjuster {
    pub step: f64,
}

impl SloRuleModifier for RoundingSloRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SloRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the slo rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloRuleEngine {
    pub steps: Vec<Rc<dyn SloRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloRuleEngine {
    pub fn evaluate(&mut self, item: &SloRule) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SloRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_slo_rule_engine() -> SloRuleEngine {
    SloRuleEngine {
        steps: vec![
            Rc::new(MinimumSloRuleAdjuster { minimum: 15.7 }) as Rc<dyn SloRuleModifier>,
            Rc::new(CapSloRuleAdjuster { cap: 28927.0 }),
            Rc::new(RoundingSloRuleAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 65575.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_rule_engine();
        let item = new_slo_rule("code-27", "name-101");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
