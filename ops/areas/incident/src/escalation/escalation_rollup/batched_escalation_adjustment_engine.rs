use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the batched escalation adjustment calculation.
pub trait BatchedEscalationAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BatchedEscalationAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitBatchedEscalationAdjustmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BatchedEscalationAdjustmentFactor for UnitBatchedEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &BatchedEscalationAdjustment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBatchedEscalationAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BatchedEscalationAdjustmentFactor for FlaggedBatchedEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BatchedEscalationAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBatchedEscalationAdjustmentAdjuster {
    pub minimum: f64,
}

impl BatchedEscalationAdjustmentFactor for MinimumBatchedEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BatchedEscalationAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBatchedEscalationAdjustmentAdjuster {
    pub percent: f64,
}

impl BatchedEscalationAdjustmentFactor for PercentBatchedEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BatchedEscalationAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBatchedEscalationAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BatchedEscalationAdjustmentFactor for TieredBatchedEscalationAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BatchedEscalationAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the batched escalation adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BatchedEscalationAdjustmentEngine {
    pub steps: Vec<Rc<dyn BatchedEscalationAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BatchedEscalationAdjustmentEngine {
    pub fn run(&mut self, item: &BatchedEscalationAdjustment) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[BatchedEscalationAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_batched_escalation_adjustment_engine(flags: &flags::Registry) -> BatchedEscalationAdjustmentEngine {
    BatchedEscalationAdjustmentEngine {
        steps: vec![
            Rc::new(UnitBatchedEscalationAdjustmentAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.18), ("north".to_string(), 0.96)]),
            }) as Rc<dyn BatchedEscalationAdjustmentFactor>,
            Rc::new(FlaggedBatchedEscalationAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "batched-escalation-adjustment-pending".to_string(),
                percent: 9.7,
            }),
            Rc::new(MinimumBatchedEscalationAdjustmentAdjuster { minimum: 38.2 }),
            Rc::new(PercentBatchedEscalationAdjustmentAdjuster { percent: 10.8 }),
            Rc::new(TieredBatchedEscalationAdjustmentAdjuster {
                bounds: vec![3405.0, 9407.0],
                rates: vec![1.06, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 92865.9,
    }
}
