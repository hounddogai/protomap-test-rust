use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze record calculation.
pub trait FreezeRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezeRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezeRecordAdjuster {
    pub minimum: f64,
}

impl FreezeRecordStep for MinimumFreezeRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezeRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRecordLimitOffsetAdjuster {
    pub weight: f64,
}

impl FreezeRecordStep for FreezeRecordLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &FreezeRecord, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezeRecordStep for TieredFreezeRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezeRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the freeze record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezeRecordEngine {
    pub steps: Vec<Rc<dyn FreezeRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezeRecordEngine {
    pub fn run(&mut self, item: &FreezeRecord) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[FreezeRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_freeze_record_engine() -> FreezeRecordEngine {
    FreezeRecordEngine {
        steps: vec![
            Rc::new(MinimumFreezeRecordAdjuster { minimum: 42.8 }) as Rc<dyn FreezeRecordStep>,
            Rc::new(FreezeRecordLimitOffsetAdjuster { weight: 1.8 }),
            Rc::new(TieredFreezeRecordAdjuster {
                bounds: vec![191.0, 2060.0, 2164.0],
                rates: vec![0.81, 0.88, 1.08],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 16485.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_record_engine();
        let item = new_freeze_record("code-849", "label-111");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
