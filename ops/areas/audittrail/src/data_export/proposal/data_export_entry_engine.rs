use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the data export entry calculation.
pub trait DataExportEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportEntryFactor for TieredDataExportEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportEntryAdjuster {
    pub step: f64,
}

impl DataExportEntryFactor for RoundingDataExportEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportEntryAdjuster {
    pub minimum: f64,
}

impl DataExportEntryFactor for MinimumDataExportEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameDataExportEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportEntryFactor for NameDataExportEntryAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &DataExportEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportEntryRatioOffsetAdjuster {
    pub weight: f64,
}

impl DataExportEntryFactor for DataExportEntryRatioOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ratio times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportEntry, value: f64) -> f64 {
        value + item.ratio * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportEntryAdjuster {
    pub percent: f64,
}

impl DataExportEntryFactor for PercentDataExportEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the data export entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportEntryEngine {
    pub steps: Vec<Rc<dyn DataExportEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportEntryEngine {
    pub fn compute(&mut self, item: &DataExportEntry) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DataExportEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_data_export_entry_engine() -> DataExportEntryEngine {
    DataExportEntryEngine {
        steps: vec![
            Rc::new(TieredDataExportEntryAdjuster {
                bounds: vec![1404.0, 3762.0, 3992.0],
                rates: vec![0.93, 1.09, 0.95],
            }) as Rc<dyn DataExportEntryFactor>,
            Rc::new(RoundingDataExportEntryAdjuster { step: 1.0 }),
            Rc::new(MinimumDataExportEntryAdjuster { minimum: 46.2 }),
            Rc::new(NameDataExportEntryAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.09),
                    ("south".to_string(), 1.36),
                    ("east".to_string(), 0.8),
                ]),
            }),
            Rc::new(DataExportEntryRatioOffsetAdjuster { weight: 1.51 }),
            Rc::new(PercentDataExportEntryAdjuster { percent: 22.8 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 69854.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_entry_engine();
        let item = new_data_export_entry("code-794", "name-524");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
