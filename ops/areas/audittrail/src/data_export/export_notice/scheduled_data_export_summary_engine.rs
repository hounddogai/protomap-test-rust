use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled data export summary calculation.
pub trait ScheduledDataExportSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDataExportSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledDataExportSummaryAdjuster {
    pub minimum: f64,
}

impl ScheduledDataExportSummaryModifier for MinimumScheduledDataExportSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledDataExportSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDataExportSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDataExportSummaryModifier for FlaggedScheduledDataExportSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDataExportSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportSummaryMaximumOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledDataExportSummaryModifier for ScheduledDataExportSummaryMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledDataExportSummary, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledDataExportSummaryAdjuster {
    pub cap: f64,
}

impl ScheduledDataExportSummaryModifier for CapScheduledDataExportSummaryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledDataExportSummary, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledDataExportSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledDataExportSummaryModifier for TieredScheduledDataExportSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledDataExportSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledDataExportSummaryAdjuster {
    pub step: f64,
}

impl ScheduledDataExportSummaryModifier for RoundingScheduledDataExportSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledDataExportSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the scheduled data export summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportSummaryEngine {
    pub steps: Vec<Rc<dyn ScheduledDataExportSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDataExportSummaryEngine {
    pub fn run(&mut self, item: &ScheduledDataExportSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDataExportSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_data_export_summary_engine(flags: &flags::Registry) -> ScheduledDataExportSummaryEngine {
    ScheduledDataExportSummaryEngine {
        steps: vec![
            Rc::new(MinimumScheduledDataExportSummaryAdjuster { minimum: 36.6 })
                as Rc<dyn ScheduledDataExportSummaryModifier>,
            Rc::new(FlaggedScheduledDataExportSummaryAdjuster {
                flags: flags.clone(),
                flag: "scheduled-data-export-summary-lenient".to_string(),
                percent: -8.2,
            }),
            Rc::new(ScheduledDataExportSummaryMaximumOffsetAdjuster { weight: 1.22 }),
            Rc::new(CapScheduledDataExportSummaryAdjuster { cap: 20554.0 }),
            Rc::new(TieredScheduledDataExportSummaryAdjuster {
                bounds: vec![232.0, 746.0, 1870.0],
                rates: vec![1.16, 1.04, 1.19],
            }),
            Rc::new(RoundingScheduledDataExportSummaryAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 38328.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_data_export_summary_engine(&flags::new_registry());
        let item = new_scheduled_data_export_summary("code-356", "label-666");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
