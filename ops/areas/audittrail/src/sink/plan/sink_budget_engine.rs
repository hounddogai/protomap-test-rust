use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the sink budget calculation.
pub trait SinkBudgetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkBudget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategorySinkBudgetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkBudgetAdjuster for CategorySinkBudgetAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &SinkBudget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkBudgetAdjuster {
    pub percent: f64,
}

impl SinkBudgetAdjuster for PercentSinkBudgetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkBudget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSinkBudgetAdjuster {
    pub cap: f64,
}

impl SinkBudgetAdjuster for CapSinkBudgetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SinkBudget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkBudgetLimitOffsetAdjuster {
    pub weight: f64,
}

impl SinkBudgetAdjuster for SinkBudgetLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &SinkBudget, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

/// Runs the sink budget steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkBudgetEngine {
    pub steps: Vec<Rc<dyn SinkBudgetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkBudgetEngine {
    pub fn evaluate(&mut self, item: &SinkBudget) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[SinkBudget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_sink_budget_engine() -> SinkBudgetEngine {
    SinkBudgetEngine {
        steps: vec![
            Rc::new(CategorySinkBudgetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.92), ("west".to_string(), 0.78)]),
            }) as Rc<dyn SinkBudgetAdjuster>,
            Rc::new(PercentSinkBudgetAdjuster { percent: 1.8 }),
            Rc::new(CapSinkBudgetAdjuster { cap: 1988.0 }),
            Rc::new(SinkBudgetLimitOffsetAdjuster { weight: 0.37 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 67180.8,
    }
}
