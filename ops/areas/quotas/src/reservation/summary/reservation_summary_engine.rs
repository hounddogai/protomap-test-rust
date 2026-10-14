use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the reservation summary calculation.
pub trait ReservationSummaryAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReservationSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelReservationSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReservationSummaryAdjuster for LabelReservationSummaryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &ReservationSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReservationSummaryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReservationSummaryAdjuster for FlaggedReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReservationSummary, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReservationSummaryAdjuster {
    pub minimum: f64,
}

impl ReservationSummaryAdjuster for MinimumReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReservationSummary, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReservationSummaryAdjuster {
    pub percent: f64,
}

impl ReservationSummaryAdjuster for PercentReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReservationSummary, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingReservationSummaryAdjuster {
    pub step: f64,
}

impl ReservationSummaryAdjuster for RoundingReservationSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ReservationSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the reservation summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReservationSummaryEngine {
    pub steps: Vec<Rc<dyn ReservationSummaryAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReservationSummaryEngine {
    pub fn compute(&mut self, item: &ReservationSummary) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ReservationSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_reservation_summary_engine(flags: &flags::Registry) -> ReservationSummaryEngine {
    ReservationSummaryEngine {
        steps: vec![
            Rc::new(LabelReservationSummaryAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.08), ("south".to_string(), 1.21)]),
            }) as Rc<dyn ReservationSummaryAdjuster>,
            Rc::new(FlaggedReservationSummaryAdjuster {
                flags: flags.clone(),
                flag: "reservation-summary-manual".to_string(),
                percent: -5.8,
            }),
            Rc::new(MinimumReservationSummaryAdjuster { minimum: 31.3 }),
            Rc::new(PercentReservationSummaryAdjuster { percent: 3.4 }),
            Rc::new(RoundingReservationSummaryAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 1072.3,
    }
}
