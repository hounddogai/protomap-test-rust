use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the seasonal data export reservation calculation.
pub trait SeasonalDataExportReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalDataExportReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelSeasonalDataExportReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalDataExportReservationStep for LabelSeasonalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &SeasonalDataExportReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSeasonalDataExportReservationAdjuster {
    pub percent: f64,
}

impl SeasonalDataExportReservationStep for PercentSeasonalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SeasonalDataExportReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDataExportReservationWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalDataExportReservationStep for SeasonalDataExportReservationWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalDataExportReservation, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalDataExportReservationAdjuster {
    pub minimum: f64,
}

impl SeasonalDataExportReservationStep for MinimumSeasonalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalDataExportReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalDataExportReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalDataExportReservationStep for FlaggedSeasonalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalDataExportReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapSeasonalDataExportReservationAdjuster {
    pub cap: f64,
}

impl SeasonalDataExportReservationStep for CapSeasonalDataExportReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &SeasonalDataExportReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the seasonal data export reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalDataExportReservationEngine {
    pub steps: Vec<Rc<dyn SeasonalDataExportReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalDataExportReservationEngine {
    pub fn price(&mut self, item: &SeasonalDataExportReservation) -> f64 {
        let mut current = item.net_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalDataExportReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalDataExportReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_seasonal_data_export_reservation_engine(flags: &flags::Registry) -> SeasonalDataExportReservationEngine {
    SeasonalDataExportReservationEngine {
        steps: vec![
            Rc::new(LabelSeasonalDataExportReservationAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.24),
                    ("east".to_string(), 1.0),
                    ("north".to_string(), 0.73),
                ]),
            }) as Rc<dyn SeasonalDataExportReservationStep>,
            Rc::new(PercentSeasonalDataExportReservationAdjuster { percent: 9.0 }),
            Rc::new(SeasonalDataExportReservationWindowDaysOffsetAdjuster { weight: 1.07 }),
            Rc::new(MinimumSeasonalDataExportReservationAdjuster { minimum: 30.0 }),
            Rc::new(FlaggedSeasonalDataExportReservationAdjuster {
                flags: flags.clone(),
                flag: "seasonal-data-export-reservation-primary".to_string(),
                percent: 9.9,
            }),
            Rc::new(CapSeasonalDataExportReservationAdjuster { cap: 2503.0 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 52892.7,
    }
}
