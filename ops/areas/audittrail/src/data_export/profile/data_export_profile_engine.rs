use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export profile calculation.
pub trait DataExportProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportProfileStep for TieredDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportProfileAdjuster {
    pub cap: f64,
}

impl DataExportProfileStep for CapDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportProfileAdjuster {
    pub step: f64,
}

impl DataExportProfileStep for RoundingDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportProfile, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportProfileStep for FlaggedDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportProfileAdjuster {
    pub percent: f64,
}

impl DataExportProfileStep for PercentDataExportProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the data export profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportProfileEngine {
    pub steps: Vec<Rc<dyn DataExportProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportProfileEngine {
    pub fn evaluate(&mut self, item: &DataExportProfile) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[DataExportProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_profile_engine(flags: &flags::Registry) -> DataExportProfileEngine {
    DataExportProfileEngine {
        steps: vec![
            Rc::new(TieredDataExportProfileAdjuster {
                bounds: vec![2842.0, 4755.0, 4767.0, 7327.0],
                rates: vec![0.94, 1.07, 1.08, 0.89],
            }) as Rc<dyn DataExportProfileStep>,
            Rc::new(CapDataExportProfileAdjuster { cap: 6405.0 }),
            Rc::new(RoundingDataExportProfileAdjuster { step: 0.25 }),
            Rc::new(FlaggedDataExportProfileAdjuster {
                flags: flags.clone(),
                flag: "data-export-profile-manual".to_string(),
                percent: 0.8,
            }),
            Rc::new(PercentDataExportProfileAdjuster { percent: 18.7 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 95052.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_profile_engine(&flags::new_registry());
        let item = new_data_export_profile("code-425", "name-783");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
