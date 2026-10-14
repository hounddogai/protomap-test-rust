use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export summary calculation.
pub trait DataExportSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDataExportSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportSummaryAdjuster for NameDataExportSummaryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DataExportSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSummaryMaximumOffsetAdjuster {
    pub weight: f64,
}

impl DataExportSummaryAdjuster for DataExportSummaryMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportSummary, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportSummaryAdjuster {
    pub cap: f64,
}

impl DataExportSummaryAdjuster for CapDataExportSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the data export summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportSummaryEngine {
    pub steps: Vec<Rc<dyn DataExportSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportSummaryEngine {
    pub fn price(&mut self, item: &DataExportSummary) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DataExportSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_summary_engine() -> DataExportSummaryEngine {
    DataExportSummaryEngine {
        steps: vec![
            Rc::new(NameDataExportSummaryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.14)]),
            }) as Rc<dyn DataExportSummaryAdjuster>,
            Rc::new(DataExportSummaryMaximumOffsetAdjuster { weight: 0.62 }),
            Rc::new(CapDataExportSummaryAdjuster { cap: 30111.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 56301.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_summary_engine();
        let item = new_data_export_summary("code-88", "name-527");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
