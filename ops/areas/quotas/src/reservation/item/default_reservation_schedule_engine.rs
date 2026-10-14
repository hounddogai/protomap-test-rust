use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default reservation schedule calculation.
pub trait DefaultReservationScheduleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultReservationSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDefaultReservationScheduleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl DefaultReservationScheduleAdjuster for TieredDefaultReservationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &DefaultReservationSchedule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultReservationScheduleAdjuster {
    pub minimum: f64,
}

impl DefaultReservationScheduleAdjuster for MinimumDefaultReservationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultReservationSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultReservationScheduleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultReservationScheduleAdjuster for FlaggedDefaultReservationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultReservationSchedule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReservationScheduleMaximumOffsetAdjuster {
    pub weight: f64,
}

impl DefaultReservationScheduleAdjuster for DefaultReservationScheduleMaximumOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus maximum times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultReservationSchedule, value: f64) -> f64 {
        value + item.maximum * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultReservationScheduleAdjuster {
    pub cap: f64,
}

impl DefaultReservationScheduleAdjuster for CapDefaultReservationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultReservationSchedule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultReservationScheduleAdjuster {
    pub step: f64,
}

impl DefaultReservationScheduleAdjuster for RoundingDefaultReservationScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultReservationSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default reservation schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultReservationScheduleEngine {
    pub steps: Vec<Rc<dyn DefaultReservationScheduleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultReservationScheduleEngine {
    pub fn run(&mut self, item: &DefaultReservationSchedule) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultReservationScheduleAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultReservationSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_reservation_schedule_engine(flags: &flags::Registry) -> DefaultReservationScheduleEngine {
    DefaultReservationScheduleEngine {
        steps: vec![
            Rc::new(TieredDefaultReservationScheduleAdjuster {
                bounds: vec![4638.0, 5785.0, 9151.0],
                rates: vec![0.84, 0.98, 0.98],
            }) as Rc<dyn DefaultReservationScheduleAdjuster>,
            Rc::new(MinimumDefaultReservationScheduleAdjuster { minimum: 5.5 }),
            Rc::new(FlaggedDefaultReservationScheduleAdjuster {
                flags: flags.clone(),
                flag: "default-reservation-schedule-seasonal".to_string(),
                percent: 4.0,
            }),
            Rc::new(DefaultReservationScheduleMaximumOffsetAdjuster { weight: 1.63 }),
            Rc::new(CapDefaultReservationScheduleAdjuster { cap: 24948.0 }),
            Rc::new(RoundingDefaultReservationScheduleAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 4140.1,
    }
}
