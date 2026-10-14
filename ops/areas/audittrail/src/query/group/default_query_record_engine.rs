use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the default query record calculation.
pub trait DefaultQueryRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultQueryRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultQueryRecordAdjuster {
    pub cap: f64,
}

impl DefaultQueryRecordFactor for CapDefaultQueryRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultQueryRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultQueryRecordAdjuster {
    pub minimum: f64,
}

impl DefaultQueryRecordFactor for MinimumDefaultQueryRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultQueryRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultQueryRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultQueryRecordFactor for TieredDefaultQueryRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultQueryRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultQueryRecordAdjuster {
    pub percent: f64,
}

impl DefaultQueryRecordFactor for PercentDefaultQueryRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultQueryRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the default query record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultQueryRecordEngine {
    pub steps: Vec<Rc<dyn DefaultQueryRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultQueryRecordEngine {
    pub fn compute(&mut self, item: &DefaultQueryRecord) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultQueryRecordFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultQueryRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_default_query_record_engine() -> DefaultQueryRecordEngine {
    DefaultQueryRecordEngine {
        steps: vec![
            Rc::new(CapDefaultQueryRecordAdjuster { cap: 48761.0 }) as Rc<dyn DefaultQueryRecordFactor>,
            Rc::new(MinimumDefaultQueryRecordAdjuster { minimum: 8.3 }),
            Rc::new(TieredDefaultQueryRecordAdjuster {
                bounds: vec![1219.0, 2882.0, 5564.0],
                rates: vec![0.88, 1.03, 1.04],
            }),
            Rc::new(PercentDefaultQueryRecordAdjuster { percent: 0.6 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 49302.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_query_record_engine();
        let item = new_default_query_record("code-540", "label-550");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
