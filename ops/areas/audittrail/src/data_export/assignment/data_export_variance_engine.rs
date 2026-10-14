use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export variance calculation.
pub trait DataExportVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportVarianceCostOffsetAdjuster {
    pub weight: f64,
}

impl DataExportVarianceStep for DataExportVarianceCostOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus cost times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportVariance, value: f64) -> f64 {
        value + item.cost * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleDataExportVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportVarianceStep for LocaleDataExportVarianceAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &DataExportVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportVarianceStep for FlaggedDataExportVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportVarianceAdjuster {
    pub step: f64,
}

impl DataExportVarianceStep for RoundingDataExportVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportVarianceEngine {
    pub steps: Vec<Rc<dyn DataExportVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportVarianceEngine {
    pub fn evaluate(&mut self, item: &DataExportVariance) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[DataExportVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_data_export_variance_engine(flags: &flags::Registry) -> DataExportVarianceEngine {
    DataExportVarianceEngine {
        steps: vec![
            Rc::new(DataExportVarianceCostOffsetAdjuster { weight: 1.62 }) as Rc<dyn DataExportVarianceStep>,
            Rc::new(LocaleDataExportVarianceAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.3),
                    ("west".to_string(), 1.37),
                    ("online".to_string(), 0.85),
                ]),
            }),
            Rc::new(FlaggedDataExportVarianceAdjuster {
                flags: flags.clone(),
                flag: "data-export-variance-scheduled".to_string(),
                percent: 6.3,
            }),
            Rc::new(RoundingDataExportVarianceAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 16084.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_variance_engine(&flags::new_registry());
        let item = new_data_export_variance("code-352", "label-188");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
