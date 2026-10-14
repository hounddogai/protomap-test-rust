use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan estimate calculation.
pub trait PlanEstimateFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanEstimateBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PlanEstimateFactor for PlanEstimateBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PlanEstimate, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePlanEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanEstimateFactor for NamePlanEstimateAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PlanEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanEstimateAdjuster {
    pub step: f64,
}

impl PlanEstimateFactor for RoundingPlanEstimateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanEstimate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the plan estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanEstimateEngine {
    pub steps: Vec<Rc<dyn PlanEstimateFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanEstimateEngine {
    pub fn price(&mut self, item: &PlanEstimate) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanEstimateFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[PlanEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_estimate_engine() -> PlanEstimateEngine {
    PlanEstimateEngine {
        steps: vec![
            Rc::new(PlanEstimateBatchSizeOffsetAdjuster { weight: 0.47 }) as Rc<dyn PlanEstimateFactor>,
            Rc::new(NamePlanEstimateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.38), ("east".to_string(), 1.2)]),
            }),
            Rc::new(RoundingPlanEstimateAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 90403.0,
    }
}
