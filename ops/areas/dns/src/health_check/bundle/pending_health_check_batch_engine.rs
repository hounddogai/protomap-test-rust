use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the pending health check batch calculation.
pub trait PendingHealthCheckBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingHealthCheckBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPendingHealthCheckBatchAdjuster {
    pub minimum: f64,
}

impl PendingHealthCheckBatchModifier for MinimumPendingHealthCheckBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PendingHealthCheckBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckBatchRevisionOffsetAdjuster {
    pub weight: f64,
}

impl PendingHealthCheckBatchModifier for PendingHealthCheckBatchRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &PendingHealthCheckBatch, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPendingHealthCheckBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingHealthCheckBatchModifier for SegmentPendingHealthCheckBatchAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PendingHealthCheckBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingHealthCheckBatchAdjuster {
    pub percent: f64,
}

impl PendingHealthCheckBatchModifier for PercentPendingHealthCheckBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingHealthCheckBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingHealthCheckBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingHealthCheckBatchModifier for FlaggedPendingHealthCheckBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingHealthCheckBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingHealthCheckBatchAdjuster {
    pub cap: f64,
}

impl PendingHealthCheckBatchModifier for CapPendingHealthCheckBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingHealthCheckBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the pending health check batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingHealthCheckBatchEngine {
    pub steps: Vec<Rc<dyn PendingHealthCheckBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingHealthCheckBatchEngine {
    pub fn compute(&mut self, item: &PendingHealthCheckBatch) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingHealthCheckBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingHealthCheckBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_health_check_batch_engine(flags: &flags::Registry) -> PendingHealthCheckBatchEngine {
    PendingHealthCheckBatchEngine {
        steps: vec![
            Rc::new(MinimumPendingHealthCheckBatchAdjuster { minimum: 49.9 })
                as Rc<dyn PendingHealthCheckBatchModifier>,
            Rc::new(PendingHealthCheckBatchRevisionOffsetAdjuster { weight: 0.35 }),
            Rc::new(SegmentPendingHealthCheckBatchAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.89),
                    ("retail".to_string(), 0.98),
                    ("north".to_string(), 1.21),
                ]),
            }),
            Rc::new(PercentPendingHealthCheckBatchAdjuster { percent: 20.7 }),
            Rc::new(FlaggedPendingHealthCheckBatchAdjuster {
                flags: flags.clone(),
                flag: "pending-health-check-batch-lenient".to_string(),
                percent: -7.3,
            }),
            Rc::new(CapPendingHealthCheckBatchAdjuster { cap: 29633.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 27180.9,
    }
}
