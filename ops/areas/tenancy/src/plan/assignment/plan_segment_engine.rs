use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan segment calculation.
pub trait PlanSegmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanSegment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanSegmentAdjuster {
    pub percent: f64,
}

impl PlanSegmentFactor for PercentPlanSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanSegment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanSegmentAdjuster {
    pub step: f64,
}

impl PlanSegmentFactor for RoundingPlanSegmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanSegment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanSegmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanSegmentFactor for FlaggedPlanSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanSegment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanSegmentAdjuster {
    pub minimum: f64,
}

impl PlanSegmentFactor for MinimumPlanSegmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanSegment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanSegmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanSegmentFactor for TieredPlanSegmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanSegment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanSegmentQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl PlanSegmentFactor for PlanSegmentQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &PlanSegment, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the plan segment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanSegmentEngine {
    pub steps: Vec<Rc<dyn PlanSegmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanSegmentEngine {
    pub fn evaluate(&mut self, item: &PlanSegment) -> f64 {
        let mut current = item.projected_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanSegmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanSegment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_segment_engine(flags: &flags::Registry) -> PlanSegmentEngine {
    PlanSegmentEngine {
        steps: vec![
            Rc::new(PercentPlanSegmentAdjuster { percent: 9.3 }) as Rc<dyn PlanSegmentFactor>,
            Rc::new(RoundingPlanSegmentAdjuster { step: 5.0 }),
            Rc::new(FlaggedPlanSegmentAdjuster {
                flags: flags.clone(),
                flag: "plan-segment-strict".to_string(),
                percent: 1.9,
            }),
            Rc::new(MinimumPlanSegmentAdjuster { minimum: 18.4 }),
            Rc::new(TieredPlanSegmentAdjuster {
                bounds: vec![1228.0, 3285.0],
                rates: vec![0.84, 0.93],
            }),
            Rc::new(PlanSegmentQuotaUnitsOffsetAdjuster { weight: 1.97 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 49107.5,
    }
}
