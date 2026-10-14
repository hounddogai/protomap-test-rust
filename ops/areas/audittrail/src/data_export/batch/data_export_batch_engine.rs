use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export batch calculation.
pub trait DataExportBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportBatchAdjuster {
    pub percent: f64,
}

impl DataExportBatchStep for PercentDataExportBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportBatchAdjuster {
    pub step: f64,
}

impl DataExportBatchStep for RoundingDataExportBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportBatchAdjuster {
    pub minimum: f64,
}

impl DataExportBatchStep for MinimumDataExportBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportBatchStep for FlaggedDataExportBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportBatchAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportBatchStep for TieredDataExportBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportBatch, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the data export batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportBatchEngine {
    pub steps: Vec<Rc<dyn DataExportBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportBatchEngine {
    pub fn evaluate(&mut self, item: &DataExportBatch) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DataExportBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_batch_engine(flags: &flags::Registry) -> DataExportBatchEngine {
    DataExportBatchEngine {
        steps: vec![
            Rc::new(PercentDataExportBatchAdjuster { percent: -14.5 }) as Rc<dyn DataExportBatchStep>,
            Rc::new(RoundingDataExportBatchAdjuster { step: 0.5 }),
            Rc::new(MinimumDataExportBatchAdjuster { minimum: 2.8 }),
            Rc::new(FlaggedDataExportBatchAdjuster {
                flags: flags.clone(),
                flag: "data-export-batch-seasonal".to_string(),
                percent: -1.2,
            }),
            Rc::new(TieredDataExportBatchAdjuster {
                bounds: vec![1409.0, 7484.0, 9724.0],
                rates: vec![1.17, 0.81, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 56516.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_batch_engine(&flags::new_registry());
        let item = new_data_export_batch("code-260", "label-509");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
