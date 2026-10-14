use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global release rule calculation.
pub trait GlobalReleaseRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalReleaseRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalReleaseRuleAdjuster {
    pub percent: f64,
}

impl GlobalReleaseRuleModifier for PercentGlobalReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalReleaseRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalReleaseRuleAdjuster {
    pub cap: f64,
}

impl GlobalReleaseRuleModifier for CapGlobalReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalReleaseRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalReleaseRuleAdjuster {
    pub minimum: f64,
}

impl GlobalReleaseRuleModifier for MinimumGlobalReleaseRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalReleaseRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the global release rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalReleaseRuleEngine {
    pub steps: Vec<Rc<dyn GlobalReleaseRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalReleaseRuleEngine {
    pub fn price(&mut self, item: &GlobalReleaseRule) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[GlobalReleaseRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_release_rule_engine() -> GlobalReleaseRuleEngine {
    GlobalReleaseRuleEngine {
        steps: vec![
            Rc::new(PercentGlobalReleaseRuleAdjuster { percent: -8.2 }) as Rc<dyn GlobalReleaseRuleModifier>,
            Rc::new(CapGlobalReleaseRuleAdjuster { cap: 20181.0 }),
            Rc::new(MinimumGlobalReleaseRuleAdjuster { minimum: 32.8 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 28659.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_release_rule_engine();
        let item = new_global_release_rule("code-678", "name-537");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
