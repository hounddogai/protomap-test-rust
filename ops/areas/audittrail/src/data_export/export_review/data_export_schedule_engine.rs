use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export schedule calculation.
pub trait DataExportScheduleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DataExportScheduleFactor for TieredDataExportScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DataExportSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportScheduleFactor for FlaggedDataExportScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportScheduleAdjuster {
    pub minimum: f64,
}

impl DataExportScheduleFactor for MinimumDataExportScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDataExportScheduleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportScheduleFactor for LabelDataExportScheduleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DataExportSchedule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportScheduleAdjuster {
    pub percent: f64,
}

impl DataExportScheduleFactor for PercentDataExportScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportSchedule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDataExportScheduleAdjuster {
    pub cap: f64,
}

impl DataExportScheduleFactor for CapDataExportScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DataExportSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the data export schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportScheduleEngine {
    pub steps: Vec<Rc<dyn DataExportScheduleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportScheduleEngine {
    pub fn price(&mut self, item: &DataExportSchedule) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[DataExportSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_schedule_engine(flags: &flags::Registry) -> DataExportScheduleEngine {
    DataExportScheduleEngine {
        steps: vec![
            Rc::new(TieredDataExportScheduleAdjuster {
                bounds: vec![2827.0, 6771.0, 9177.0],
                rates: vec![1.15, 0.87, 1.13],
            }) as Rc<dyn DataExportScheduleFactor>,
            Rc::new(FlaggedDataExportScheduleAdjuster {
                flags: flags.clone(),
                flag: "data-export-schedule-seasonal".to_string(),
                percent: -7.5,
            }),
            Rc::new(MinimumDataExportScheduleAdjuster { minimum: 2.5 }),
            Rc::new(LabelDataExportScheduleAdjuster {
                factors: BTreeMap::from([
                    ("retail".to_string(), 0.94),
                    ("central".to_string(), 1.14),
                    ("east".to_string(), 0.77),
                ]),
            }),
            Rc::new(PercentDataExportScheduleAdjuster { percent: 2.5 }),
            Rc::new(CapDataExportScheduleAdjuster { cap: 46128.0 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 27447.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_schedule_engine(&flags::new_registry());
        let item = new_data_export_schedule("code-144", "label-322");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
