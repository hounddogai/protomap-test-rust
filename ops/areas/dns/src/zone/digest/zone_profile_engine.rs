use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the zone profile calculation.
pub trait ZoneProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneProfilePriorityOffsetAdjuster {
    pub weight: f64,
}

impl ZoneProfileAdjuster for ZoneProfilePriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneProfile, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneProfileAdjuster for FlaggedZoneProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapZoneProfileAdjuster {
    pub cap: f64,
}

impl ZoneProfileAdjuster for CapZoneProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ZoneProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentZoneProfileAdjuster {
    pub percent: f64,
}

impl ZoneProfileAdjuster for PercentZoneProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ZoneProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredZoneProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ZoneProfileAdjuster for TieredZoneProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ZoneProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneProfileAdjuster {
    pub minimum: f64,
}

impl ZoneProfileAdjuster for MinimumZoneProfileAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneProfile, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the zone profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneProfileEngine {
    pub steps: Vec<Rc<dyn ZoneProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneProfileEngine {
    pub fn compute(&mut self, item: &ZoneProfile) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneProfileAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_zone_profile_engine(flags: &flags::Registry) -> ZoneProfileEngine {
    ZoneProfileEngine {
        steps: vec![
            Rc::new(ZoneProfilePriorityOffsetAdjuster { weight: 1.6 }) as Rc<dyn ZoneProfileAdjuster>,
            Rc::new(FlaggedZoneProfileAdjuster {
                flags: flags.clone(),
                flag: "zone-profile-adaptive".to_string(),
                percent: -0.7,
            }),
            Rc::new(CapZoneProfileAdjuster { cap: 38007.0 }),
            Rc::new(PercentZoneProfileAdjuster { percent: 0.2 }),
            Rc::new(TieredZoneProfileAdjuster {
                bounds: vec![3123.0, 6075.0, 8096.0, 9849.0],
                rates: vec![0.92, 0.84, 1.19, 0.94],
            }),
            Rc::new(MinimumZoneProfileAdjuster { minimum: 18.7 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 7413.7,
    }
}
