use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the release profile calculation.
pub trait ReleaseProfileStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleaseProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProfileWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl ReleaseProfileStep for ReleaseProfileWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &ReleaseProfile, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedReleaseProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ReleaseProfileStep for FlaggedReleaseProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ReleaseProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitReleaseProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ReleaseProfileStep for UnitReleaseProfileAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &ReleaseProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleaseProfileAdjuster {
    pub percent: f64,
}

impl ReleaseProfileStep for PercentReleaseProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleaseProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the release profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleaseProfileEngine {
    pub steps: Vec<Rc<dyn ReleaseProfileStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleaseProfileEngine {
    pub fn run(&mut self, item: &ReleaseProfile) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleaseProfileStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleaseProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_release_profile_engine(flags: &flags::Registry) -> ReleaseProfileEngine {
    ReleaseProfileEngine {
        steps: vec![
            Rc::new(ReleaseProfileWindowDaysOffsetAdjuster { weight: 1.82 }) as Rc<dyn ReleaseProfileStep>,
            Rc::new(FlaggedReleaseProfileAdjuster {
                flags: flags.clone(),
                flag: "release-profile-scheduled".to_string(),
                percent: 1.5,
            }),
            Rc::new(UnitReleaseProfileAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.04)]),
            }),
            Rc::new(PercentReleaseProfileAdjuster { percent: 12.3 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 27759.2,
    }
}
