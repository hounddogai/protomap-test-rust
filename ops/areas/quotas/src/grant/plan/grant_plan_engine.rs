use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant plan calculation.
pub trait GrantPlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPlanCeilingOffsetAdjuster {
    pub weight: f64,
}

impl GrantPlanStep for GrantPlanCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &GrantPlan, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantPlanAdjuster {
    pub minimum: f64,
}

impl GrantPlanStep for MinimumGrantPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGrantPlanAdjuster {
    pub cap: f64,
}

impl GrantPlanStep for CapGrantPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GrantPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the grant plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantPlanEngine {
    pub steps: Vec<Rc<dyn GrantPlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantPlanEngine {
    pub fn price(&mut self, item: &GrantPlan) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[GrantPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_grant_plan_engine() -> GrantPlanEngine {
    GrantPlanEngine {
        steps: vec![
            Rc::new(GrantPlanCeilingOffsetAdjuster { weight: 0.87 }) as Rc<dyn GrantPlanStep>,
            Rc::new(MinimumGrantPlanAdjuster { minimum: 25.6 }),
            Rc::new(CapGrantPlanAdjuster { cap: 39154.0 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 39362.6,
    }
}
