use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the lenient escalation batch calculation.
pub trait LenientEscalationBatchModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LenientEscalationBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEscalationBatchRateOffsetAdjuster {
    pub weight: f64,
}

impl LenientEscalationBatchModifier for LenientEscalationBatchRateOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rate times {}", self.weight)
    }

    fn adjust(&self, item: &LenientEscalationBatch, value: f64) -> f64 {
        value + item.rate * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLenientEscalationBatchAdjuster {
    pub step: f64,
}

impl LenientEscalationBatchModifier for RoundingLenientEscalationBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LenientEscalationBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLenientEscalationBatchAdjuster {
    pub cap: f64,
}

impl LenientEscalationBatchModifier for CapLenientEscalationBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LenientEscalationBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentLenientEscalationBatchAdjuster {
    pub percent: f64,
}

impl LenientEscalationBatchModifier for PercentLenientEscalationBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &LenientEscalationBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedLenientEscalationBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl LenientEscalationBatchModifier for FlaggedLenientEscalationBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &LenientEscalationBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the lenient escalation batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LenientEscalationBatchEngine {
    pub steps: Vec<Rc<dyn LenientEscalationBatchModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LenientEscalationBatchEngine {
    pub fn compute(&mut self, item: &LenientEscalationBatch) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[LenientEscalationBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_lenient_escalation_batch_engine(flags: &flags::Registry) -> LenientEscalationBatchEngine {
    LenientEscalationBatchEngine {
        steps: vec![
            Rc::new(LenientEscalationBatchRateOffsetAdjuster { weight: 0.45 })
                as Rc<dyn LenientEscalationBatchModifier>,
            Rc::new(RoundingLenientEscalationBatchAdjuster { step: 0.5 }),
            Rc::new(CapLenientEscalationBatchAdjuster { cap: 35436.0 }),
            Rc::new(PercentLenientEscalationBatchAdjuster { percent: 17.9 }),
            Rc::new(FlaggedLenientEscalationBatchAdjuster {
                flags: flags.clone(),
                flag: "lenient-escalation-batch-adaptive".to_string(),
                percent: -1.2,
            }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 34477.5,
    }
}
