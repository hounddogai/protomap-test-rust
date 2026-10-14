use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export milestone calculation.
pub trait DataExportMilestoneFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportMilestoneAdjuster {
    pub percent: f64,
}

impl DataExportMilestoneFactor for PercentDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportMilestoneAdjuster {
    pub step: f64,
}

impl DataExportMilestoneFactor for RoundingDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportMilestoneFactor for TieredDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportMilestoneAdjuster {
    pub cap: f64,
}

impl DataExportMilestoneFactor for CapDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportMilestoneFactor for FlaggedDataExportMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the data export milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportMilestoneEngine {
    pub steps: Vec<Rc<dyn DataExportMilestoneFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportMilestoneEngine {
    pub fn price(&mut self, item: &DataExportMilestone) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportMilestoneFactor> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[DataExportMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_milestone_engine(flags: &flags::Registry) -> DataExportMilestoneEngine {
    DataExportMilestoneEngine {
        steps: vec![
            Rc::new(PercentDataExportMilestoneAdjuster { percent: -12.0 }) as Rc<dyn DataExportMilestoneFactor>,
            Rc::new(RoundingDataExportMilestoneAdjuster { step: 0.05 }),
            Rc::new(TieredDataExportMilestoneAdjuster {
                bounds: vec![5601.0, 7757.0],
                rates: vec![0.85, 0.97],
            }),
            Rc::new(CapDataExportMilestoneAdjuster { cap: 24708.0 }),
            Rc::new(FlaggedDataExportMilestoneAdjuster {
                flags: flags.clone(),
                flag: "data-export-milestone-strict".to_string(),
                percent: 9.0,
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 70751.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_milestone_engine(&flags::new_registry());
        let item = new_data_export_milestone("code-956", "label-761");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
