use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the segment plan calculation.
pub trait SegmentPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SegmentPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSegmentPlanAdjuster {
    pub step: f64,
}

impl SegmentPlanFactor for RoundingSegmentPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SegmentPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSegmentPlanAdjuster {
    pub cap: f64,
}

impl SegmentPlanFactor for CapSegmentPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SegmentPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SegmentPlanFactor for TieredSegmentPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SegmentPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountrySegmentPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SegmentPlanFactor for CountrySegmentPlanAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &SegmentPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSegmentPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SegmentPlanFactor for FlaggedSegmentPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SegmentPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the segment plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SegmentPlanEngine {
    pub steps: Vec<Rc<dyn SegmentPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SegmentPlanEngine {
    pub fn run(&mut self, item: &SegmentPlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SegmentPlanFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SegmentPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_segment_plan_engine(flags: &flags::Registry) -> SegmentPlanEngine {
    SegmentPlanEngine {
        steps: vec![
            Rc::new(RoundingSegmentPlanAdjuster { step: 0.5 }) as Rc<dyn SegmentPlanFactor>,
            Rc::new(CapSegmentPlanAdjuster { cap: 36930.0 }),
            Rc::new(TieredSegmentPlanAdjuster {
                bounds: vec![1441.0, 4649.0, 5097.0, 8828.0],
                rates: vec![1.03, 0.96, 1.07, 1.03],
            }),
            Rc::new(CountrySegmentPlanAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.8), ("online".to_string(), 1.02)]),
            }),
            Rc::new(FlaggedSegmentPlanAdjuster {
                flags: flags.clone(),
                flag: "segment-plan-tiered".to_string(),
                percent: -6.2,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 3354.3,
    }
}
