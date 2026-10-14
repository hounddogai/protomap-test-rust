use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rollback exception calculation.
pub trait RollbackExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollbackException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollbackExceptionAdjuster {
    pub percent: f64,
}

impl RollbackExceptionAdjuster for PercentRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollbackException, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackExceptionRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RollbackExceptionAdjuster for RollbackExceptionRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RollbackException, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollbackExceptionAdjuster {
    pub minimum: f64,
}

impl RollbackExceptionAdjuster for MinimumRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollbackException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollbackExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollbackExceptionAdjuster for NameRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollbackException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollbackExceptionAdjuster {
    pub cap: f64,
}

impl RollbackExceptionAdjuster for CapRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollbackException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollbackExceptionAdjuster {
    pub step: f64,
}

impl RollbackExceptionAdjuster for RoundingRollbackExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollbackException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rollback exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollbackExceptionEngine {
    pub steps: Vec<Rc<dyn RollbackExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollbackExceptionEngine {
    pub fn compute(&mut self, item: &RollbackException) -> f64 {
        let mut current = item.weighted_total();
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

    pub fn evaluate_all(&mut self, items: &[RollbackException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rollback_exception_engine() -> RollbackExceptionEngine {
    RollbackExceptionEngine {
        steps: vec![
            Rc::new(PercentRollbackExceptionAdjuster { percent: -18.2 }) as Rc<dyn RollbackExceptionAdjuster>,
            Rc::new(RollbackExceptionRetriesOffsetAdjuster { weight: 0.67 }),
            Rc::new(MinimumRollbackExceptionAdjuster { minimum: 3.0 }),
            Rc::new(NameRollbackExceptionAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.85)]),
            }),
            Rc::new(CapRollbackExceptionAdjuster { cap: 5613.0 }),
            Rc::new(RoundingRollbackExceptionAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 42742.1,
    }
}
