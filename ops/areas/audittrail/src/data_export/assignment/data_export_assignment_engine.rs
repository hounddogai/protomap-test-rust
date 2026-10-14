use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export assignment calculation.
pub trait DataExportAssignmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportAssignmentAdjuster {
    pub percent: f64,
}

impl DataExportAssignmentFactor for PercentDataExportAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportAssignment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDataExportAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportAssignmentFactor for LabelDataExportAssignmentAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DataExportAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportAssignmentAdjuster {
    pub minimum: f64,
}

impl DataExportAssignmentFactor for MinimumDataExportAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportAssignmentAdjuster {
    pub cap: f64,
}

impl DataExportAssignmentFactor for CapDataExportAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportAssignment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the data export assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportAssignmentEngine {
    pub steps: Vec<Rc<dyn DataExportAssignmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportAssignmentEngine {
    pub fn run(&mut self, item: &DataExportAssignment) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[DataExportAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_data_export_assignment_engine() -> DataExportAssignmentEngine {
    DataExportAssignmentEngine {
        steps: vec![
            Rc::new(PercentDataExportAssignmentAdjuster { percent: 6.8 }) as Rc<dyn DataExportAssignmentFactor>,
            Rc::new(LabelDataExportAssignmentAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.72), ("east".to_string(), 1.38)]),
            }),
            Rc::new(MinimumDataExportAssignmentAdjuster { minimum: 16.6 }),
            Rc::new(CapDataExportAssignmentAdjuster { cap: 24344.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 98738.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_assignment_engine();
        let item = new_data_export_assignment("code-82", "label-514");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
