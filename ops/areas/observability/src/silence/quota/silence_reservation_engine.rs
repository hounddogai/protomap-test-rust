use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the silence reservation calculation.
pub trait SilenceReservationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SilenceReservation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedSilenceReservationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SilenceReservationStep for FlaggedSilenceReservationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SilenceReservation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentSilenceReservationAdjuster {
    pub percent: f64,
}

impl SilenceReservationStep for PercentSilenceReservationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &SilenceReservation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSilenceReservationAdjuster {
    pub minimum: f64,
}

impl SilenceReservationStep for MinimumSilenceReservationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SilenceReservation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservationRevisionOffsetAdjuster {
    pub weight: f64,
}

impl SilenceReservationStep for SilenceReservationRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &SilenceReservation, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSilenceReservationAdjuster {
    pub step: f64,
}

impl SilenceReservationStep for RoundingSilenceReservationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SilenceReservation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSilenceReservationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SilenceReservationStep for NameSilenceReservationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SilenceReservation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the silence reservation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SilenceReservationEngine {
    pub steps: Vec<Rc<dyn SilenceReservationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SilenceReservationEngine {
    pub fn run(&mut self, item: &SilenceReservation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SilenceReservationStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SilenceReservation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_silence_reservation_engine(flags: &flags::Registry) -> SilenceReservationEngine {
    SilenceReservationEngine {
        steps: vec![
            Rc::new(FlaggedSilenceReservationAdjuster {
                flags: flags.clone(),
                flag: "silence-reservation-weighted".to_string(),
                percent: 1.7,
            }) as Rc<dyn SilenceReservationStep>,
            Rc::new(PercentSilenceReservationAdjuster { percent: -10.3 }),
            Rc::new(MinimumSilenceReservationAdjuster { minimum: 7.6 }),
            Rc::new(SilenceReservationRevisionOffsetAdjuster { weight: 0.24 }),
            Rc::new(RoundingSilenceReservationAdjuster { step: 0.5 }),
            Rc::new(NameSilenceReservationAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.23),
                    ("retail".to_string(), 0.8),
                    ("south".to_string(), 0.79),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 9514.6,
    }
}
