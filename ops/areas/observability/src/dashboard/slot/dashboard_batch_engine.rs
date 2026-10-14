use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the dashboard batch calculation.
pub trait DashboardBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DashboardBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDashboardBatchAdjuster {
    pub percent: f64,
}

impl DashboardBatchStep for PercentDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DashboardBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDashboardBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DashboardBatchStep for FlaggedDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DashboardBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDashboardBatchAdjuster {
    pub cap: f64,
}

impl DashboardBatchStep for CapDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DashboardBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDashboardBatchAdjuster {
    pub minimum: f64,
}

impl DashboardBatchStep for MinimumDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DashboardBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDashboardBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DashboardBatchStep for TieredDashboardBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DashboardBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the dashboard batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DashboardBatchEngine {
    pub steps: Vec<Rc<dyn DashboardBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DashboardBatchEngine {
    pub fn evaluate(&mut self, item: &DashboardBatch) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DashboardBatchStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DashboardBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_dashboard_batch_engine(flags: &flags::Registry) -> DashboardBatchEngine {
    DashboardBatchEngine {
        steps: vec![
            Rc::new(PercentDashboardBatchAdjuster { percent: 2.4 }) as Rc<dyn DashboardBatchStep>,
            Rc::new(FlaggedDashboardBatchAdjuster {
                flags: flags.clone(),
                flag: "dashboard-batch-weighted".to_string(),
                percent: 0.5,
            }),
            Rc::new(CapDashboardBatchAdjuster { cap: 16189.0 }),
            Rc::new(MinimumDashboardBatchAdjuster { minimum: 33.9 }),
            Rc::new(TieredDashboardBatchAdjuster {
                bounds: vec![5557.0, 6655.0, 6920.0, 9977.0],
                rates: vec![1.17, 1.08, 1.09, 1.06],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 16218.0,
    }
}
