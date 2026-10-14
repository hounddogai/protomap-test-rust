use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the zone assignment calculation.
pub trait ZoneAssignmentModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ZoneAssignment, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedZoneAssignmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ZoneAssignmentModifier for FlaggedZoneAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ZoneAssignment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumZoneAssignmentAdjuster {
    pub minimum: f64,
}

impl ZoneAssignmentModifier for MinimumZoneAssignmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ZoneAssignment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceZoneAssignmentAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ZoneAssignmentModifier for SourceZoneAssignmentAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &ZoneAssignment, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAssignmentRetriesOffsetAdjuster {
    pub weight: f64,
}

impl ZoneAssignmentModifier for ZoneAssignmentRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &ZoneAssignment, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

/// Runs the zone assignment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ZoneAssignmentEngine {
    pub steps: Vec<Rc<dyn ZoneAssignmentModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ZoneAssignmentEngine {
    pub fn run(&mut self, item: &ZoneAssignment) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ZoneAssignmentModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ZoneAssignment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_zone_assignment_engine(flags: &flags::Registry) -> ZoneAssignmentEngine {
    ZoneAssignmentEngine {
        steps: vec![
            Rc::new(FlaggedZoneAssignmentAdjuster {
                flags: flags.clone(),
                flag: "zone-assignment-rolling".to_string(),
                percent: -0.6,
            }) as Rc<dyn ZoneAssignmentModifier>,
            Rc::new(MinimumZoneAssignmentAdjuster { minimum: 28.3 }),
            Rc::new(SourceZoneAssignmentAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.76)]),
            }),
            Rc::new(ZoneAssignmentRetriesOffsetAdjuster { weight: 0.62 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 75337.2,
    }
}
