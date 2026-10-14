use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled data export threshold calculation.
pub trait ScheduledDataExportThresholdAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledDataExportThreshold, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledDataExportThresholdAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledDataExportThresholdAdjuster for FlaggedScheduledDataExportThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledDataExportThreshold, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredScheduledDataExportThresholdAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ScheduledDataExportThresholdAdjuster for TieredScheduledDataExportThresholdAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ScheduledDataExportThreshold, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDataExportThresholdGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledDataExportThresholdAdjuster for ScheduledDataExportThresholdGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledDataExportThreshold, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the scheduled data export threshold steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledDataExportThresholdEngine {
    pub steps: Vec<Rc<dyn ScheduledDataExportThresholdAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledDataExportThresholdEngine {
    pub fn price(&mut self, item: &ScheduledDataExportThreshold) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledDataExportThreshold]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_scheduled_data_export_threshold_engine(flags: &flags::Registry) -> ScheduledDataExportThresholdEngine {
    ScheduledDataExportThresholdEngine {
        steps: vec![
            Rc::new(FlaggedScheduledDataExportThresholdAdjuster {
                flags: flags.clone(),
                flag: "scheduled-data-export-threshold-strict".to_string(),
                percent: 9.6,
            }) as Rc<dyn ScheduledDataExportThresholdAdjuster>,
            Rc::new(TieredScheduledDataExportThresholdAdjuster {
                bounds: vec![2195.0, 2383.0, 4713.0, 9273.0],
                rates: vec![1.18, 0.87, 1.02, 0.91],
            }),
            Rc::new(ScheduledDataExportThresholdGraceDaysOffsetAdjuster { weight: 0.02 }),
        ],
        trace: Vec::new(),
        floor: 0.1,
        ceiling: 79338.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_data_export_threshold_engine(&flags::new_registry());
        let item = new_scheduled_data_export_threshold("code-10", "label-880");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
