use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the weighted timeline plan calculation.
pub trait WeightedTimelinePlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTimelinePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelinePlanAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl WeightedTimelinePlanStep for WeightedTimelinePlanAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedTimelinePlan, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTimelinePlanAdjuster {
    pub minimum: f64,
}

impl WeightedTimelinePlanStep for MinimumWeightedTimelinePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTimelinePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedTimelinePlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedTimelinePlanStep for FlaggedWeightedTimelinePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedTimelinePlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedTimelinePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedTimelinePlanStep for TieredWeightedTimelinePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedTimelinePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelWeightedTimelinePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl WeightedTimelinePlanStep for LabelWeightedTimelinePlanAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &WeightedTimelinePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedTimelinePlanAdjuster {
    pub cap: f64,
}

impl WeightedTimelinePlanStep for CapWeightedTimelinePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedTimelinePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the weighted timeline plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTimelinePlanEngine {
    pub steps: Vec<Rc<dyn WeightedTimelinePlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTimelinePlanEngine {
    pub fn price(&mut self, item: &WeightedTimelinePlan) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedTimelinePlanStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTimelinePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_timeline_plan_engine(flags: &flags::Registry) -> WeightedTimelinePlanEngine {
    WeightedTimelinePlanEngine {
        steps: vec![
            Rc::new(WeightedTimelinePlanAttemptsOffsetAdjuster { weight: 1.66 }) as Rc<dyn WeightedTimelinePlanStep>,
            Rc::new(MinimumWeightedTimelinePlanAdjuster { minimum: 44.5 }),
            Rc::new(FlaggedWeightedTimelinePlanAdjuster {
                flags: flags.clone(),
                flag: "weighted-timeline-plan-lenient".to_string(),
                percent: 7.5,
            }),
            Rc::new(TieredWeightedTimelinePlanAdjuster {
                bounds: vec![1969.0, 4715.0, 7998.0],
                rates: vec![1.0, 1.18, 1.17],
            }),
            Rc::new(LabelWeightedTimelinePlanAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.8),
                    ("retail".to_string(), 0.73),
                    ("east".to_string(), 0.81),
                ]),
            }),
            Rc::new(CapWeightedTimelinePlanAdjuster { cap: 31843.0 }),
        ],
        trace: Vec::new(),
        floor: 1.9,
        ceiling: 43648.6,
    }
}
