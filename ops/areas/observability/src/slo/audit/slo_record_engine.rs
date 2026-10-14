use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the slo record calculation.
pub trait SloRecordStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SloRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSloRecordAdjuster {
    pub minimum: f64,
}

impl SloRecordStep for MinimumSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SloRecord, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSloRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SloRecordStep for TieredSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SloRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSloRecordAdjuster {
    pub percent: f64,
}

impl SloRecordStep for PercentSloRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SloRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the slo record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SloRecordEngine {
    pub steps: Vec<Rc<dyn SloRecordStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SloRecordEngine {
    pub fn compute(&mut self, item: &SloRecord) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SloRecordStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SloRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_slo_record_engine() -> SloRecordEngine {
    SloRecordEngine {
        steps: vec![
            Rc::new(MinimumSloRecordAdjuster { minimum: 22.1 }) as Rc<dyn SloRecordStep>,
            Rc::new(TieredSloRecordAdjuster {
                bounds: vec![5780.0, 7949.0],
                rates: vec![0.87, 1.05],
            }),
            Rc::new(PercentSloRecordAdjuster { percent: -11.2 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 21855.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_slo_record_engine();
        let item = new_slo_record("code-714", "title-212");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
