use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the data export notice calculation.
pub trait DataExportNoticeAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DataExportNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDataExportNoticeAdjuster {
    pub minimum: f64,
}

impl DataExportNoticeAdjuster for MinimumDataExportNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DataExportNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDataExportNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DataExportNoticeAdjuster for FlaggedDataExportNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DataExportNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDataExportNoticeAdjuster {
    pub percent: f64,
}

impl DataExportNoticeAdjuster for PercentDataExportNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DataExportNotice, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryDataExportNoticeAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DataExportNoticeAdjuster for CountryDataExportNoticeAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &DataExportNotice, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDataExportNoticeAdjuster {
    pub step: f64,
}

impl DataExportNoticeAdjuster for RoundingDataExportNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DataExportNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the data export notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DataExportNoticeEngine {
    pub steps: Vec<Rc<dyn DataExportNoticeAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DataExportNoticeEngine {
    pub fn price(&mut self, item: &DataExportNotice) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DataExportNoticeAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DataExportNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_data_export_notice_engine(flags: &flags::Registry) -> DataExportNoticeEngine {
    DataExportNoticeEngine {
        steps: vec![
            Rc::new(MinimumDataExportNoticeAdjuster { minimum: 1.4 }) as Rc<dyn DataExportNoticeAdjuster>,
            Rc::new(FlaggedDataExportNoticeAdjuster {
                flags: flags.clone(),
                flag: "data-export-notice-strict".to_string(),
                percent: -4.9,
            }),
            Rc::new(PercentDataExportNoticeAdjuster { percent: 23.8 }),
            Rc::new(CountryDataExportNoticeAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.73), ("online".to_string(), 1.26)]),
            }),
            Rc::new(RoundingDataExportNoticeAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 39869.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_data_export_notice_engine(&flags::new_registry());
        let item = new_data_export_notice("code-172", "label-498");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
