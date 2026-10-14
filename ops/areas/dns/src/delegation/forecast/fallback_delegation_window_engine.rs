use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the fallback delegation window calculation.
pub trait FallbackDelegationWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackDelegationWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFallbackDelegationWindowAdjuster {
    pub cap: f64,
}

impl FallbackDelegationWindowAdjuster for CapFallbackDelegationWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FallbackDelegationWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackDelegationWindowAdjuster {
    pub percent: f64,
}

impl FallbackDelegationWindowAdjuster for PercentFallbackDelegationWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackDelegationWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackDelegationWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackDelegationWindowAdjuster for FlaggedFallbackDelegationWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackDelegationWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelFallbackDelegationWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackDelegationWindowAdjuster for LabelFallbackDelegationWindowAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &FallbackDelegationWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback delegation window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackDelegationWindowEngine {
    pub steps: Vec<Rc<dyn FallbackDelegationWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackDelegationWindowEngine {
    pub fn compute(&mut self, item: &FallbackDelegationWindow) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackDelegationWindowAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackDelegationWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_delegation_window_engine(flags: &flags::Registry) -> FallbackDelegationWindowEngine {
    FallbackDelegationWindowEngine {
        steps: vec![
            Rc::new(CapFallbackDelegationWindowAdjuster { cap: 12967.0 }) as Rc<dyn FallbackDelegationWindowAdjuster>,
            Rc::new(PercentFallbackDelegationWindowAdjuster { percent: 17.6 }),
            Rc::new(FlaggedFallbackDelegationWindowAdjuster {
                flags: flags.clone(),
                flag: "fallback-delegation-window-primary".to_string(),
                percent: 3.0,
            }),
            Rc::new(LabelFallbackDelegationWindowAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.0)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 96536.7,
    }
}
