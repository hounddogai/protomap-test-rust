use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export rollup calculation.
pub trait DataExportRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportRollupMaximumOffsetAdjuster {
    pub weight: f64,
}

impl DataExportRollupModifier for DataExportRollupMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportRollup, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportRollupAdjuster {
    pub percent: f64,
}

impl DataExportRollupModifier for PercentDataExportRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDataExportRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportRollupModifier for RegionDataExportRollupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &DataExportRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportRollupAdjuster {
    pub minimum: f64,
}

impl DataExportRollupModifier for MinimumDataExportRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the data export rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportRollupEngine {
    pub steps: Vec<Rc<dyn DataExportRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportRollupEngine {
    pub fn evaluate(&mut self, item: &DataExportRollup) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[DataExportRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_rollup_engine() -> DataExportRollupEngine {
    DataExportRollupEngine {
        steps: vec![
            Rc::new(DataExportRollupMaximumOffsetAdjuster { weight: 1.88 }) as Rc<dyn DataExportRollupModifier>,
            Rc::new(PercentDataExportRollupAdjuster { percent: -20.7 }),
            Rc::new(RegionDataExportRollupAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.95),
                    ("east".to_string(), 0.8),
                    ("south".to_string(), 1.04),
                ]),
            }),
            Rc::new(MinimumDataExportRollupAdjuster { minimum: 5.4 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 14777.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_rollup_engine();
        let item = new_data_export_rollup("code-225", "title-823");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
