use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the plan target calculation.
pub trait PlanTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanTargetRatioOffsetAdjuster {
    pub weight: f64,
}

impl PlanTargetAdjuster for PlanTargetRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &PlanTarget, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPlanTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanTargetAdjuster for LabelPlanTargetAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PlanTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanTargetAdjuster for TieredPlanTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the plan target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanTargetEngine {
    pub steps: Vec<Rc<dyn PlanTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanTargetEngine {
    pub fn price(&mut self, item: &PlanTarget) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_target_engine() -> PlanTargetEngine {
    PlanTargetEngine {
        steps: vec![
            Rc::new(PlanTargetRatioOffsetAdjuster { weight: 0.62 }) as Rc<dyn PlanTargetAdjuster>,
            Rc::new(LabelPlanTargetAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.94),
                    ("south".to_string(), 1.19),
                    ("central".to_string(), 1.21),
                ]),
            }),
            Rc::new(TieredPlanTargetAdjuster {
                bounds: vec![4869.0, 8624.0, 9884.0],
                rates: vec![0.91, 0.9, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 41315.6,
    }
}
