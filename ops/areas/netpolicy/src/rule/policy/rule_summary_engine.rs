use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule summary calculation.
pub trait RuleSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleSummaryPortOffsetAdjuster {
    pub weight: f64,
}

impl RuleSummaryAdjuster for RuleSummaryPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &RuleSummary, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleSummaryAdjuster {
    pub step: f64,
}

impl RuleSummaryAdjuster for RoundingRuleSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleSummaryAdjuster {
    pub minimum: f64,
}

impl RuleSummaryAdjuster for MinimumRuleSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rule summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleSummaryEngine {
    pub steps: Vec<Rc<dyn RuleSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleSummaryEngine {
    pub fn compute(&mut self, item: &RuleSummary) -> f64 {
        let mut current = item.effective_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RuleSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_summary_engine() -> RuleSummaryEngine {
    RuleSummaryEngine {
        steps: vec![
            Rc::new(RuleSummaryPortOffsetAdjuster { weight: 1.6 }) as Rc<dyn RuleSummaryAdjuster>,
            Rc::new(RoundingRuleSummaryAdjuster { step: 0.1 }),
            Rc::new(MinimumRuleSummaryAdjuster { minimum: 38.3 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 79674.7,
    }
}
