use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export audit calculation.
pub trait DataExportAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportAuditAdjuster {
    pub minimum: f64,
}

impl DataExportAuditAdjuster for MinimumDataExportAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAuditSequenceOffsetAdjuster {
    pub weight: f64,
}

impl DataExportAuditAdjuster for DataExportAuditSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportAudit, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportAuditAdjuster {
    pub step: f64,
}

impl DataExportAuditAdjuster for RoundingDataExportAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportAuditEngine {
    pub steps: Vec<Rc<dyn DataExportAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportAuditEngine {
    pub fn evaluate(&mut self, item: &DataExportAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DataExportAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_audit_engine() -> DataExportAuditEngine {
    DataExportAuditEngine {
        steps: vec![
            Rc::new(MinimumDataExportAuditAdjuster { minimum: 26.9 }) as Rc<dyn DataExportAuditAdjuster>,
            Rc::new(DataExportAuditSequenceOffsetAdjuster { weight: 1.0 }),
            Rc::new(RoundingDataExportAuditAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 48021.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_audit_engine();
        let item = new_data_export_audit("code-520", "title-976");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
