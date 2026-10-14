use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the binding milestone calculation.
pub trait BindingMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingMilestoneAdjuster {
    pub percent: f64,
}

impl BindingMilestoneStep for PercentBindingMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingMilestoneAdjuster {
    pub minimum: f64,
}

impl BindingMilestoneStep for MinimumBindingMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingMilestoneQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl BindingMilestoneStep for BindingMilestoneQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &BindingMilestone, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionBindingMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingMilestoneStep for RegionBindingMilestoneAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &BindingMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingMilestoneAdjuster {
    pub step: f64,
}

impl BindingMilestoneStep for RoundingBindingMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the binding milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingMilestoneEngine {
    pub steps: Vec<Rc<dyn BindingMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingMilestoneEngine {
    pub fn compute(&mut self, item: &BindingMilestone) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_binding_milestone_engine() -> BindingMilestoneEngine {
    BindingMilestoneEngine {
        steps: vec![
            Rc::new(PercentBindingMilestoneAdjuster { percent: -17.3 }) as Rc<dyn BindingMilestoneStep>,
            Rc::new(MinimumBindingMilestoneAdjuster { minimum: 26.3 }),
            Rc::new(BindingMilestoneQuotaUnitsOffsetAdjuster { weight: 1.46 }),
            Rc::new(RegionBindingMilestoneAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.23)]),
            }),
            Rc::new(RoundingBindingMilestoneAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 20357.4,
    }
}
