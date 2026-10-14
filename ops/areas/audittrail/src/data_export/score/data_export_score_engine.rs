use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export score calculation.
pub trait DataExportScoreFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScoreCeilingOffsetAdjuster {
    pub weight: f64,
}

impl DataExportScoreFactor for DataExportScoreCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &DataExportScore, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportScoreFactor for TieredDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportScoreAdjuster {
    pub cap: f64,
}

impl DataExportScoreFactor for CapDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportScoreAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportScoreFactor for FlaggedDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportScore, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportScoreAdjuster {
    pub minimum: f64,
}

impl DataExportScoreFactor for MinimumDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportScoreAdjuster {
    pub step: f64,
}

impl DataExportScoreFactor for RoundingDataExportScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportScoreEngine {
    pub steps: Vec<Rc<dyn DataExportScoreFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportScoreEngine {
    pub fn compute(&mut self, item: &DataExportScore) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[DataExportScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_data_export_score_engine(flags: &flags::Registry) -> DataExportScoreEngine {
    DataExportScoreEngine {
        steps: vec![
            Rc::new(DataExportScoreCeilingOffsetAdjuster { weight: 0.22 }) as Rc<dyn DataExportScoreFactor>,
            Rc::new(TieredDataExportScoreAdjuster {
                bounds: vec![1956.0, 2657.0],
                rates: vec![0.96, 0.98],
            }),
            Rc::new(CapDataExportScoreAdjuster { cap: 15347.0 }),
            Rc::new(FlaggedDataExportScoreAdjuster {
                flags: flags.clone(),
                flag: "data-export-score-strict".to_string(),
                percent: 0.7,
            }),
            Rc::new(MinimumDataExportScoreAdjuster { minimum: 6.3 }),
            Rc::new(RoundingDataExportScoreAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 27638.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_score_engine(&flags::new_registry());
        let item = new_data_export_score("code-684", "label-804");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
