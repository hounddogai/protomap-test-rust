use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tiered measurement record calculation.
pub trait TieredMeasurementRecordFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TieredMeasurementRecord, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTieredMeasurementRecordAdjuster {
    pub percent: f64,
}

impl TieredMeasurementRecordFactor for PercentTieredMeasurementRecordAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TieredMeasurementRecord, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementRecordWeightOffsetAdjuster {
    pub weight: f64,
}

impl TieredMeasurementRecordFactor for TieredMeasurementRecordWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &TieredMeasurementRecord, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTieredMeasurementRecordAdjuster {
    pub cap: f64,
}

impl TieredMeasurementRecordFactor for CapTieredMeasurementRecordAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TieredMeasurementRecord, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTieredMeasurementRecordAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TieredMeasurementRecordFactor for TieredTieredMeasurementRecordAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TieredMeasurementRecord, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the tiered measurement record steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TieredMeasurementRecordEngine {
    pub steps: Vec<Rc<dyn TieredMeasurementRecordFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TieredMeasurementRecordEngine {
    pub fn price(&mut self, item: &TieredMeasurementRecord) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TieredMeasurementRecordFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TieredMeasurementRecord]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tiered_measurement_record_engine() -> TieredMeasurementRecordEngine {
    TieredMeasurementRecordEngine {
        steps: vec![
            Rc::new(PercentTieredMeasurementRecordAdjuster { percent: 4.0 }) as Rc<dyn TieredMeasurementRecordFactor>,
            Rc::new(TieredMeasurementRecordWeightOffsetAdjuster { weight: 0.42 }),
            Rc::new(CapTieredMeasurementRecordAdjuster { cap: 48559.0 }),
            Rc::new(TieredTieredMeasurementRecordAdjuster {
                bounds: vec![2332.0, 7813.0],
                rates: vec![1.08, 0.97],
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 20867.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tiered_measurement_record_engine();
        let item = new_tiered_measurement_record("code-223", "title-155");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
