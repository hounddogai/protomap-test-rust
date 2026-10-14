use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the sink reservation calculation.
pub trait SinkReservationModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SinkReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSinkReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SinkReservationModifier for FlaggedSinkReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SinkReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSinkReservationAdjuster {
    pub step: f64,
}

impl SinkReservationModifier for RoundingSinkReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SinkReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceSinkReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SinkReservationModifier for SourceSinkReservationAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &SinkReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSinkReservationAdjuster {
    pub percent: f64,
}

impl SinkReservationModifier for PercentSinkReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SinkReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the sink reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SinkReservationEngine {
    pub steps: Vec<Rc<dyn SinkReservationModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SinkReservationEngine {
    pub fn price(&mut self, item: &SinkReservation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[SinkReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_sink_reservation_engine(flags: &flags::Registry) -> SinkReservationEngine {
    SinkReservationEngine {
        steps: vec![
            Rc::new(FlaggedSinkReservationAdjuster {
                flags: flags.clone(),
                flag: "sink-reservation-fallback".to_string(),
                percent: 7.2,
            }) as Rc<dyn SinkReservationModifier>,
            Rc::new(RoundingSinkReservationAdjuster { step: 1.0 }),
            Rc::new(SourceSinkReservationAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 1.09),
                    ("north".to_string(), 1.11),
                    ("west".to_string(), 1.06),
                ]),
            }),
            Rc::new(PercentSinkReservationAdjuster { percent: -22.1 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 20549.8,
    }
}
