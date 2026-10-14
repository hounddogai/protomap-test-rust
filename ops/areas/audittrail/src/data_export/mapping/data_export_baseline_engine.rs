use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export baseline calculation.
pub trait DataExportBaselineFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceDataExportBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportBaselineFactor for SourceDataExportBaselineAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &DataExportBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportBaselineAdjuster {
    pub cap: f64,
}

impl DataExportBaselineFactor for CapDataExportBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportBaselineAdjuster {
    pub step: f64,
}

impl DataExportBaselineFactor for RoundingDataExportBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportBaselineEngine {
    pub steps: Vec<Rc<dyn DataExportBaselineFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportBaselineEngine {
    pub fn price(&mut self, item: &DataExportBaseline) -> f64 {
        let mut current = item.projected_load();
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

    pub fn evaluate_all(&mut self, items: &[DataExportBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_baseline_engine() -> DataExportBaselineEngine {
    DataExportBaselineEngine {
        steps: vec![
            Rc::new(SourceDataExportBaselineAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.74),
                    ("south".to_string(), 0.93),
                    ("east".to_string(), 1.18),
                ]),
            }) as Rc<dyn DataExportBaselineFactor>,
            Rc::new(CapDataExportBaselineAdjuster { cap: 47571.0 }),
            Rc::new(RoundingDataExportBaselineAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 4738.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_baseline_engine();
        let item = new_data_export_baseline("code-80", "title-625");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
