use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the legacy isolation record calculation.
pub trait LegacyIsolationRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &LegacyIsolationRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredLegacyIsolationRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl LegacyIsolationRecordStep for TieredLegacyIsolationRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &LegacyIsolationRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingLegacyIsolationRecordAdjuster {
    pub step: f64,
}

impl LegacyIsolationRecordStep for RoundingLegacyIsolationRecordAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &LegacyIsolationRecord, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapLegacyIsolationRecordAdjuster {
    pub cap: f64,
}

impl LegacyIsolationRecordStep for CapLegacyIsolationRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &LegacyIsolationRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the legacy isolation record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct LegacyIsolationRecordEngine {
    pub steps: Vec<Rc<dyn LegacyIsolationRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl LegacyIsolationRecordEngine {
    pub fn run(&mut self, item: &LegacyIsolationRecord) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn LegacyIsolationRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[LegacyIsolationRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_legacy_isolation_record_engine() -> LegacyIsolationRecordEngine {
    LegacyIsolationRecordEngine {
        steps: vec![
            Rc::new(TieredLegacyIsolationRecordAdjuster {
                bounds: vec![3306.0, 4536.0, 8255.0, 9942.0],
                rates: vec![1.14, 1.0, 0.83, 1.03],
            }) as Rc<dyn LegacyIsolationRecordStep>,
            Rc::new(RoundingLegacyIsolationRecordAdjuster { step: 5.0 }),
            Rc::new(CapLegacyIsolationRecordAdjuster { cap: 9874.0 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 23756.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_legacy_isolation_record_engine();
        let item = new_legacy_isolation_record("code-232", "title-58");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
