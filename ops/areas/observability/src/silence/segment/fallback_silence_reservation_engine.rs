use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback silence reservation calculation.
pub trait FallbackSilenceReservationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackSilenceReservation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackSilenceReservationAdjuster {
    pub minimum: f64,
}

impl FallbackSilenceReservationFactor for MinimumFallbackSilenceReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackSilenceReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackSilenceReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackSilenceReservationFactor for FlaggedFallbackSilenceReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackSilenceReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackSilenceReservationAdjuster {
    pub cap: f64,
}

impl FallbackSilenceReservationFactor for CapFallbackSilenceReservationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackSilenceReservation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the fallback silence reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackSilenceReservationEngine {
    pub steps: Vec<Rc<dyn FallbackSilenceReservationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackSilenceReservationEngine {
    pub fn evaluate(&mut self, item: &FallbackSilenceReservation) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[FallbackSilenceReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_fallback_silence_reservation_engine(flags: &flags::Registry) -> FallbackSilenceReservationEngine {
    FallbackSilenceReservationEngine {
        steps: vec![
            Rc::new(MinimumFallbackSilenceReservationAdjuster { minimum: 27.8 })
                as Rc<dyn FallbackSilenceReservationFactor>,
            Rc::new(FlaggedFallbackSilenceReservationAdjuster {
                flags: flags.clone(),
                flag: "fallback-silence-reservation-manual".to_string(),
                percent: -7.1,
            }),
            Rc::new(CapFallbackSilenceReservationAdjuster { cap: 1988.0 }),
        ],
        trace: Vec::new(),
        floor: 2.5,
        ceiling: 26304.8,
    }
}
