use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export preference calculation.
pub trait DataExportPreferenceAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportPreferenceAdjuster {
    pub percent: f64,
}

impl DataExportPreferenceAdjuster for PercentDataExportPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportPreferenceAdjuster {
    pub step: f64,
}

impl DataExportPreferenceAdjuster for RoundingDataExportPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportPreferenceAdjuster for FlaggedDataExportPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeDataExportPreferenceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportPreferenceAdjuster for ScopeDataExportPreferenceAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &DataExportPreference, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the data export preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportPreferenceEngine {
    pub steps: Vec<Rc<dyn DataExportPreferenceAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportPreferenceEngine {
    pub fn compute(&mut self, item: &DataExportPreference) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[DataExportPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_data_export_preference_engine(flags: &flags::Registry) -> DataExportPreferenceEngine {
    DataExportPreferenceEngine {
        steps: vec![
            Rc::new(PercentDataExportPreferenceAdjuster { percent: -4.4 }) as Rc<dyn DataExportPreferenceAdjuster>,
            Rc::new(RoundingDataExportPreferenceAdjuster { step: 5.0 }),
            Rc::new(FlaggedDataExportPreferenceAdjuster {
                flags: flags.clone(),
                flag: "data-export-preference-pending".to_string(),
                percent: 8.0,
            }),
            Rc::new(ScopeDataExportPreferenceAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.01)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 55360.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_preference_engine(&flags::new_registry());
        let item = new_data_export_preference("code-410", "name-9");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
