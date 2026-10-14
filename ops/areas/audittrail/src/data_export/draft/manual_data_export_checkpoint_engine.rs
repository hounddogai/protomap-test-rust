use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the manual data export checkpoint calculation.
pub trait ManualDataExportCheckpointModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualDataExportCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapManualDataExportCheckpointAdjuster {
    pub cap: f64,
}

impl ManualDataExportCheckpointModifier for CapManualDataExportCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ManualDataExportCheckpoint, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualDataExportCheckpointAdjuster {
    pub minimum: f64,
}

impl ManualDataExportCheckpointModifier for MinimumManualDataExportCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualDataExportCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualDataExportCheckpointAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualDataExportCheckpointModifier for TieredManualDataExportCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualDataExportCheckpoint, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the manual data export checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualDataExportCheckpointEngine {
    pub steps: Vec<Rc<dyn ManualDataExportCheckpointModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualDataExportCheckpointEngine {
    pub fn price(&mut self, item: &ManualDataExportCheckpoint) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ManualDataExportCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_data_export_checkpoint_engine() -> ManualDataExportCheckpointEngine {
    ManualDataExportCheckpointEngine {
        steps: vec![
            Rc::new(CapManualDataExportCheckpointAdjuster { cap: 42797.0 })
                as Rc<dyn ManualDataExportCheckpointModifier>,
            Rc::new(MinimumManualDataExportCheckpointAdjuster { minimum: 12.1 }),
            Rc::new(TieredManualDataExportCheckpointAdjuster {
                bounds: vec![7213.0, 9045.0],
                rates: vec![0.88, 1.03],
            }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 9868.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_data_export_checkpoint_engine();
        let item = new_manual_data_export_checkpoint("code-596", "name-536");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
