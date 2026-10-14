use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan rollup calculation.
pub trait PlanRollupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanRollupAdjuster {
    pub minimum: f64,
}

impl PlanRollupAdjuster for MinimumPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanRollupAdjuster for FlaggedPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanRollupAdjuster {
    pub cap: f64,
}

impl PlanRollupAdjuster for CapPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanRollupGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl PlanRollupAdjuster for PlanRollupGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &PlanRollup, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanRollupAdjuster {
    pub step: f64,
}

impl PlanRollupAdjuster for RoundingPlanRollupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanRollup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryPlanRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanRollupAdjuster for CategoryPlanRollupAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &PlanRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the plan rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanRollupEngine {
    pub steps: Vec<Rc<dyn PlanRollupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanRollupEngine {
    pub fn compute(&mut self, item: &PlanRollup) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanRollupAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_rollup_engine(flags: &flags::Registry) -> PlanRollupEngine {
    PlanRollupEngine {
        steps: vec![
            Rc::new(MinimumPlanRollupAdjuster { minimum: 27.7 }) as Rc<dyn PlanRollupAdjuster>,
            Rc::new(FlaggedPlanRollupAdjuster {
                flags: flags.clone(),
                flag: "plan-rollup-regional".to_string(),
                percent: -6.3,
            }),
            Rc::new(CapPlanRollupAdjuster { cap: 7893.0 }),
            Rc::new(PlanRollupGraceDaysOffsetAdjuster { weight: 1.66 }),
            Rc::new(RoundingPlanRollupAdjuster { step: 0.1 }),
            Rc::new(CategoryPlanRollupAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.3),
                    ("online".to_string(), 1.24),
                    ("east".to_string(), 0.9),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 9704.5,
    }
}
