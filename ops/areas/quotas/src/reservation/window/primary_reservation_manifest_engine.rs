use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary reservation manifest calculation.
pub trait PrimaryReservationManifestStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryReservationManifest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryReservationManifestAdjuster {
    pub step: f64,
}

impl PrimaryReservationManifestStep for RoundingPrimaryReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryReservationManifest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryReservationManifestAdjuster {
    pub cap: f64,
}

impl PrimaryReservationManifestStep for CapPrimaryReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryReservationManifest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryReservationManifestVersionOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryReservationManifestStep for PrimaryReservationManifestVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryReservationManifest, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryReservationManifestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryReservationManifestStep for FlaggedPrimaryReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryReservationManifest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryReservationManifestAdjuster {
    pub minimum: f64,
}

impl PrimaryReservationManifestStep for MinimumPrimaryReservationManifestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryReservationManifest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryReservationManifestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryReservationManifestStep for SourcePrimaryReservationManifestAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryReservationManifest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary reservation manifest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryReservationManifestEngine {
    pub steps: Vec<Rc<dyn PrimaryReservationManifestStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryReservationManifestEngine {
    pub fn evaluate(&mut self, item: &PrimaryReservationManifest) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryReservationManifest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_reservation_manifest_engine(flags: &flags::Registry) -> PrimaryReservationManifestEngine {
    PrimaryReservationManifestEngine {
        steps: vec![
            Rc::new(RoundingPrimaryReservationManifestAdjuster { step: 0.25 })
                as Rc<dyn PrimaryReservationManifestStep>,
            Rc::new(CapPrimaryReservationManifestAdjuster { cap: 48697.0 }),
            Rc::new(PrimaryReservationManifestVersionOffsetAdjuster { weight: 1.1 }),
            Rc::new(FlaggedPrimaryReservationManifestAdjuster {
                flags: flags.clone(),
                flag: "primary-reservation-manifest-scheduled".to_string(),
                percent: 4.7,
            }),
            Rc::new(MinimumPrimaryReservationManifestAdjuster { minimum: 28.7 }),
            Rc::new(SourcePrimaryReservationManifestAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.83)]),
            }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 7074.5,
    }
}
