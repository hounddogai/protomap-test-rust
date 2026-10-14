use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the manual measurement batch calculation.
pub trait ManualMeasurementBatchFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualMeasurementBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualMeasurementBatchAdjuster {
    pub cap: f64,
}

impl ManualMeasurementBatchFactor for CapManualMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualMeasurementBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualMeasurementBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualMeasurementBatchFactor for FlaggedManualMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualMeasurementBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualMeasurementBatchVersionOffsetAdjuster {
    pub weight: f64,
}

impl ManualMeasurementBatchFactor for ManualMeasurementBatchVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &ManualMeasurementBatch, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingManualMeasurementBatchAdjuster {
    pub step: f64,
}

impl ManualMeasurementBatchFactor for RoundingManualMeasurementBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ManualMeasurementBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the manual measurement batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualMeasurementBatchEngine {
    pub steps: Vec<Rc<dyn ManualMeasurementBatchFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualMeasurementBatchEngine {
    pub fn price(&mut self, item: &ManualMeasurementBatch) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[ManualMeasurementBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_measurement_batch_engine(flags: &flags::Registry) -> ManualMeasurementBatchEngine {
    ManualMeasurementBatchEngine {
        steps: vec![
            Rc::new(CapManualMeasurementBatchAdjuster { cap: 35918.0 }) as Rc<dyn ManualMeasurementBatchFactor>,
            Rc::new(FlaggedManualMeasurementBatchAdjuster {
                flags: flags.clone(),
                flag: "manual-measurement-batch-global".to_string(),
                percent: -9.8,
            }),
            Rc::new(ManualMeasurementBatchVersionOffsetAdjuster { weight: 0.14 }),
            Rc::new(RoundingManualMeasurementBatchAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 41495.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_measurement_batch_engine(&flags::new_registry());
        let item = new_manual_measurement_batch("code-876", "label-417");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
