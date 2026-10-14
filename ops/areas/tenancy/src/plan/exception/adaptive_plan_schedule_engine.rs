use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the adaptive plan schedule calculation.
pub trait AdaptivePlanScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AdaptivePlanSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAdaptivePlanScheduleAdjuster {
    pub minimum: f64,
}

impl AdaptivePlanScheduleFactor for MinimumAdaptivePlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AdaptivePlanSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAdaptivePlanScheduleAdjuster {
    pub cap: f64,
}

impl AdaptivePlanScheduleFactor for CapAdaptivePlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AdaptivePlanSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAdaptivePlanScheduleAdjuster {
    pub percent: f64,
}

impl AdaptivePlanScheduleFactor for PercentAdaptivePlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AdaptivePlanSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAdaptivePlanScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AdaptivePlanScheduleFactor for TieredAdaptivePlanScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AdaptivePlanSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceAdaptivePlanScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl AdaptivePlanScheduleFactor for SourceAdaptivePlanScheduleAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &AdaptivePlanSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the adaptive plan schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AdaptivePlanScheduleEngine {
    pub steps: Vec<Rc<dyn AdaptivePlanScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AdaptivePlanScheduleEngine {
    pub fn price(&mut self, item: &AdaptivePlanSchedule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AdaptivePlanScheduleFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AdaptivePlanSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_adaptive_plan_schedule_engine() -> AdaptivePlanScheduleEngine {
    AdaptivePlanScheduleEngine {
        steps: vec![
            Rc::new(MinimumAdaptivePlanScheduleAdjuster { minimum: 1.5 }) as Rc<dyn AdaptivePlanScheduleFactor>,
            Rc::new(CapAdaptivePlanScheduleAdjuster { cap: 41032.0 }),
            Rc::new(PercentAdaptivePlanScheduleAdjuster { percent: 10.7 }),
            Rc::new(TieredAdaptivePlanScheduleAdjuster {
                bounds: vec![555.0, 3587.0, 5437.0],
                rates: vec![1.02, 0.86, 1.01],
            }),
            Rc::new(SourceAdaptivePlanScheduleAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.05), ("east".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 30628.6,
    }
}
