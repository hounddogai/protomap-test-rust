use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export override calculation.
pub trait DataExportOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportOverrideAdjuster {
    pub cap: f64,
}

impl DataExportOverrideModifier for CapDataExportOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportOverrideAdjuster {
    pub step: f64,
}

impl DataExportOverrideModifier for RoundingDataExportOverrideAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportOverride, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportOverrideAdjuster {
    pub minimum: f64,
}

impl DataExportOverrideModifier for MinimumDataExportOverrideAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportOverride, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the data export override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportOverrideEngine {
    pub steps: Vec<Rc<dyn DataExportOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportOverrideEngine {
    pub fn run(&mut self, item: &DataExportOverride) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportOverrideModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_data_export_override_engine() -> DataExportOverrideEngine {
    DataExportOverrideEngine {
        steps: vec![
            Rc::new(CapDataExportOverrideAdjuster { cap: 16961.0 }) as Rc<dyn DataExportOverrideModifier>,
            Rc::new(RoundingDataExportOverrideAdjuster { step: 1.0 }),
            Rc::new(MinimumDataExportOverrideAdjuster { minimum: 18.5 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 39163.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_override_engine();
        let item = new_data_export_override("code-318", "name-841");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
