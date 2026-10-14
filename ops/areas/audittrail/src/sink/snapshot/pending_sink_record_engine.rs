use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending sink record calculation.
pub trait PendingSinkRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingSinkRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingSinkRecordAdjuster {
    pub step: f64,
}

impl PendingSinkRecordStep for RoundingPendingSinkRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingSinkRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingSinkRecordTtlSecondsOffsetAdjuster {
    pub weight: f64,
}

impl PendingSinkRecordStep for PendingSinkRecordTtlSecondsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ttl seconds times {}", self.weight)
    }

    fn adjust(&self, item: &PendingSinkRecord, value: f64) -> f64 {
        value + (item.ttl_seconds as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePendingSinkRecordAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingSinkRecordStep for NamespacePendingSinkRecordAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &PendingSinkRecord, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending sink record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingSinkRecordEngine {
    pub steps: Vec<Rc<dyn PendingSinkRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingSinkRecordEngine {
    pub fn evaluate(&mut self, item: &PendingSinkRecord) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PendingSinkRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PendingSinkRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_sink_record_engine() -> PendingSinkRecordEngine {
    PendingSinkRecordEngine {
        steps: vec![
            Rc::new(RoundingPendingSinkRecordAdjuster { step: 0.1 }) as Rc<dyn PendingSinkRecordStep>,
            Rc::new(PendingSinkRecordTtlSecondsOffsetAdjuster { weight: 1.49 }),
            Rc::new(NamespacePendingSinkRecordAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.31),
                    ("west".to_string(), 0.77),
                    ("east".to_string(), 1.39),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 48907.9,
    }
}
