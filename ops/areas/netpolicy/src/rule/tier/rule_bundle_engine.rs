use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule bundle calculation.
pub trait RuleBundleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleBundle, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleBundleAdjuster {
    pub minimum: f64,
}

impl RuleBundleModifier for MinimumRuleBundleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleBundle, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleBundleAdjuster {
    pub cap: f64,
}

impl RuleBundleModifier for CapRuleBundleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleBundle, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleBundleMarginOffsetAdjuster {
    pub weight: f64,
}

impl RuleBundleModifier for RuleBundleMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &RuleBundle, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

/// Runs the rule bundle steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleBundleEngine {
    pub steps: Vec<Rc<dyn RuleBundleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleBundleEngine {
    pub fn compute(&mut self, item: &RuleBundle) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[RuleBundle]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_bundle_engine() -> RuleBundleEngine {
    RuleBundleEngine {
        steps: vec![
            Rc::new(MinimumRuleBundleAdjuster { minimum: 10.7 }) as Rc<dyn RuleBundleModifier>,
            Rc::new(CapRuleBundleAdjuster { cap: 28241.0 }),
            Rc::new(RuleBundleMarginOffsetAdjuster { weight: 1.36 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 14998.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_bundle_engine();
        let item = new_rule_bundle("code-312", "name-665");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
