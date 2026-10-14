use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict reservation draft calculation.
pub trait StrictReservationDraftStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictReservationDraft, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictReservationDraftBurstOffsetAdjuster {
    pub weight: f64,
}

impl StrictReservationDraftStep for StrictReservationDraftBurstOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus burst times {}", self.weight)
    }

    fn adjust(&self, item: &StrictReservationDraft, value: f64) -> f64 {
        value + (item.burst as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictReservationDraftAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictReservationDraftStep for FlaggedStrictReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictReservationDraft, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictReservationDraftAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictReservationDraftStep for TieredStrictReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictReservationDraft, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentStrictReservationDraftAdjuster {
    pub percent: f64,
}

impl StrictReservationDraftStep for PercentStrictReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &StrictReservationDraft, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictReservationDraftAdjuster {
    pub minimum: f64,
}

impl StrictReservationDraftStep for MinimumStrictReservationDraftAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictReservationDraft, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict reservation draft steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictReservationDraftEngine {
    pub steps: Vec<Rc<dyn StrictReservationDraftStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictReservationDraftEngine {
    pub fn run(&mut self, item: &StrictReservationDraft) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[StrictReservationDraft]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_reservation_draft_engine(flags: &flags::Registry) -> StrictReservationDraftEngine {
    StrictReservationDraftEngine {
        steps: vec![
            Rc::new(StrictReservationDraftBurstOffsetAdjuster { weight: 1.1 }) as Rc<dyn StrictReservationDraftStep>,
            Rc::new(FlaggedStrictReservationDraftAdjuster {
                flags: flags.clone(),
                flag: "strict-reservation-draft-lenient".to_string(),
                percent: 9.0,
            }),
            Rc::new(TieredStrictReservationDraftAdjuster {
                bounds: vec![3242.0, 5265.0, 7254.0],
                rates: vec![1.19, 1.09, 0.97],
            }),
            Rc::new(PercentStrictReservationDraftAdjuster { percent: -18.4 }),
            Rc::new(MinimumStrictReservationDraftAdjuster { minimum: 39.1 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 85282.7,
    }
}
