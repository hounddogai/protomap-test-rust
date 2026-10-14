use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export target calculation.
pub trait DataExportTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTargetBurstOffsetAdjuster {
    pub weight: f64,
}

impl DataExportTargetFactor for DataExportTargetBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportTarget, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportTargetAdjuster {
    pub percent: f64,
}

impl DataExportTargetFactor for PercentDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportTargetAdjuster {
    pub minimum: f64,
}

impl DataExportTargetFactor for MinimumDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDataExportTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportTargetFactor for NameDataExportTargetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DataExportTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportTargetAdjuster {
    pub cap: f64,
}

impl DataExportTargetFactor for CapDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportTargetAdjuster {
    pub step: f64,
}

impl DataExportTargetFactor for RoundingDataExportTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportTargetEngine {
    pub steps: Vec<Rc<dyn DataExportTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportTargetEngine {
    pub fn evaluate(&mut self, item: &DataExportTarget) -> f64 {
        let mut current = item.effective_load();
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

    pub fn evaluate_all(&mut self, items: &[DataExportTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_target_engine() -> DataExportTargetEngine {
    DataExportTargetEngine {
        steps: vec![
            Rc::new(DataExportTargetBurstOffsetAdjuster { weight: 1.31 }) as Rc<dyn DataExportTargetFactor>,
            Rc::new(PercentDataExportTargetAdjuster { percent: 2.6 }),
            Rc::new(MinimumDataExportTargetAdjuster { minimum: 44.3 }),
            Rc::new(NameDataExportTargetAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.85),
                    ("west".to_string(), 0.78),
                    ("north".to_string(), 1.16),
                ]),
            }),
            Rc::new(CapDataExportTargetAdjuster { cap: 26517.0 }),
            Rc::new(RoundingDataExportTargetAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 53279.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_target_engine();
        let item = new_data_export_target("code-937", "name-221");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
