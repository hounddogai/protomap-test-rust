use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the seasonal egress batch calculation.
pub trait SeasonalEgressBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalEgressBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalEgressBatchAdjuster {
    pub percent: f64,
}

impl SeasonalEgressBatchModifier for PercentSeasonalEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalEgressBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleSeasonalEgressBatchAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalEgressBatchModifier for LocaleSeasonalEgressBatchAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &SeasonalEgressBatch, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressBatchFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalEgressBatchModifier for SeasonalEgressBatchFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalEgressBatch, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalEgressBatchAdjuster {
    pub cap: f64,
}

impl SeasonalEgressBatchModifier for CapSeasonalEgressBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalEgressBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the seasonal egress batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalEgressBatchEngine {
    pub steps: Vec<Rc<dyn SeasonalEgressBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalEgressBatchEngine {
    pub fn evaluate(&mut self, item: &SeasonalEgressBatch) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalEgressBatchModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalEgressBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_seasonal_egress_batch_engine() -> SeasonalEgressBatchEngine {
    SeasonalEgressBatchEngine {
        steps: vec![
            Rc::new(PercentSeasonalEgressBatchAdjuster { percent: -19.2 }) as Rc<dyn SeasonalEgressBatchModifier>,
            Rc::new(LocaleSeasonalEgressBatchAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.92)]),
            }),
            Rc::new(SeasonalEgressBatchFloorValueOffsetAdjuster { weight: 1.58 }),
            Rc::new(CapSeasonalEgressBatchAdjuster { cap: 13453.0 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 22394.2,
    }
}
