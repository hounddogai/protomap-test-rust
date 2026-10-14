use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the reservation entry calculation.
pub trait ReservationEntryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationEntryAdjuster {
    pub minimum: f64,
}

impl ReservationEntryAdjuster for MinimumReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationEntryAdjuster for FlaggedReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReservationEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationEntryAdjuster for LabelReservationEntryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ReservationEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationEntryAdjuster {
    pub percent: f64,
}

impl ReservationEntryAdjuster for PercentReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapReservationEntryAdjuster {
    pub cap: f64,
}

impl ReservationEntryAdjuster for CapReservationEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ReservationEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the reservation entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationEntryEngine {
    pub steps: Vec<Rc<dyn ReservationEntryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationEntryEngine {
    pub fn price(&mut self, item: &ReservationEntry) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReservationEntryAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReservationEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_reservation_entry_engine(flags: &flags::Registry) -> ReservationEntryEngine {
    ReservationEntryEngine {
        steps: vec![
            Rc::new(MinimumReservationEntryAdjuster { minimum: 39.4 }) as Rc<dyn ReservationEntryAdjuster>,
            Rc::new(FlaggedReservationEntryAdjuster {
                flags: flags.clone(),
                flag: "reservation-entry-strict".to_string(),
                percent: -1.1,
            }),
            Rc::new(LabelReservationEntryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.29)]),
            }),
            Rc::new(PercentReservationEntryAdjuster { percent: -15.2 }),
            Rc::new(CapReservationEntryAdjuster { cap: 12449.0 }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 87673.6,
    }
}
