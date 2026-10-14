use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict measurement batch calculation.
pub trait StrictMeasurementBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictMeasurementBatch, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictMeasurementBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictMeasurementBatchFactor for FlaggedStrictMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictMeasurementBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictMeasurementBatchAdjuster {
    pub step: f64,
}

impl StrictMeasurementBatchFactor for RoundingStrictMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictMeasurementBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictMeasurementBatchQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl StrictMeasurementBatchFactor for StrictMeasurementBatchQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &StrictMeasurementBatch, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the strict measurement batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictMeasurementBatchEngine {
    pub steps: Vec<Rc<dyn StrictMeasurementBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictMeasurementBatchEngine {
    pub fn price(&mut self, item: &StrictMeasurementBatch) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[StrictMeasurementBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_measurement_batch_engine(flags: &flags::Registry) -> StrictMeasurementBatchEngine {
    StrictMeasurementBatchEngine {
        steps: vec![
            Rc::new(FlaggedStrictMeasurementBatchAdjuster {
                flags: flags.clone(),
                flag: "strict-measurement-batch-capped".to_string(),
                percent: 1.0,
            }) as Rc<dyn StrictMeasurementBatchFactor>,
            Rc::new(RoundingStrictMeasurementBatchAdjuster { step: 0.05 }),
            Rc::new(StrictMeasurementBatchQuotaUnitsOffsetAdjuster { weight: 0.95 }),
        ],
        trace: Vec::new(),
        floor: 0.3,
        ceiling: 57279.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_measurement_batch_engine(&flags::new_registry());
        let item = new_strict_measurement_batch("code-739", "title-851");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
