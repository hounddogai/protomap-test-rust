use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the nonce record calculation.
pub trait NonceRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &NonceRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapNonceRecordAdjuster {
    pub cap: f64,
}

impl NonceRecordStep for CapNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &NonceRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentNonceRecordAdjuster {
    pub percent: f64,
}

impl NonceRecordStep for PercentNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &NonceRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRecordBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl NonceRecordStep for NonceRecordBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &NonceRecord, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingNonceRecordAdjuster {
    pub step: f64,
}

impl NonceRecordStep for RoundingNonceRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &NonceRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the nonce record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct NonceRecordEngine {
    pub steps: Vec<Rc<dyn NonceRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl NonceRecordEngine {
    pub fn compute(&mut self, item: &NonceRecord) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn NonceRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[NonceRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_nonce_record_engine() -> NonceRecordEngine {
    NonceRecordEngine {
        steps: vec![
            Rc::new(CapNonceRecordAdjuster { cap: 2917.0 }) as Rc<dyn NonceRecordStep>,
            Rc::new(PercentNonceRecordAdjuster { percent: 25.0 }),
            Rc::new(NonceRecordBatchSizeOffsetAdjuster { weight: 1.54 }),
            Rc::new(RoundingNonceRecordAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 22090.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_nonce_record_engine();
        let item = new_nonce_record("code-454", "name-983");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
