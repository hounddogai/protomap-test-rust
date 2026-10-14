use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the canary milestone calculation.
pub trait CanaryMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CanaryMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapCanaryMilestoneAdjuster {
    pub cap: f64,
}

impl CanaryMilestoneModifier for CapCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &CanaryMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentCanaryMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CanaryMilestoneModifier for SegmentCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &CanaryMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCanaryMilestoneAdjuster {
    pub percent: f64,
}

impl CanaryMilestoneModifier for PercentCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CanaryMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCanaryMilestoneAdjuster {
    pub minimum: f64,
}

impl CanaryMilestoneModifier for MinimumCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CanaryMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCanaryMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CanaryMilestoneModifier for TieredCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CanaryMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCanaryMilestoneAdjuster {
    pub step: f64,
}

impl CanaryMilestoneModifier for RoundingCanaryMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CanaryMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the canary milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CanaryMilestoneEngine {
    pub steps: Vec<Rc<dyn CanaryMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CanaryMilestoneEngine {
    pub fn run(&mut self, item: &CanaryMilestone) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CanaryMilestoneModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[CanaryMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_canary_milestone_engine() -> CanaryMilestoneEngine {
    CanaryMilestoneEngine {
        steps: vec![
            Rc::new(CapCanaryMilestoneAdjuster { cap: 49467.0 }) as Rc<dyn CanaryMilestoneModifier>,
            Rc::new(SegmentCanaryMilestoneAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.24),
                    ("west".to_string(), 1.21),
                    ("south".to_string(), 1.29),
                ]),
            }),
            Rc::new(PercentCanaryMilestoneAdjuster { percent: 24.7 }),
            Rc::new(MinimumCanaryMilestoneAdjuster { minimum: 3.4 }),
            Rc::new(TieredCanaryMilestoneAdjuster {
                bounds: vec![3190.0, 5050.0, 8319.0],
                rates: vec![0.97, 0.89, 0.91],
            }),
            Rc::new(RoundingCanaryMilestoneAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 13752.3,
    }
}
