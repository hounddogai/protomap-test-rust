use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the policy window calculation.
pub trait PolicyWindowStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyWindowAdjuster {
    pub cap: f64,
}

impl PolicyWindowStep for CapPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyWindowEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PolicyWindowStep for PolicyWindowEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyWindow, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyWindowAdjuster {
    pub percent: f64,
}

impl PolicyWindowStep for PercentPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyWindowAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyWindowStep for FlaggedPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyWindow, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePolicyWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyWindowStep for ScopePolicyWindowAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PolicyWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyWindowAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyWindowStep for TieredPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyWindow, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the policy window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyWindowEngine {
    pub steps: Vec<Rc<dyn PolicyWindowStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyWindowEngine {
    pub fn compute(&mut self, item: &PolicyWindow) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyWindowStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_policy_window_engine(flags: &flags::Registry) -> PolicyWindowEngine {
    PolicyWindowEngine {
        steps: vec![
            Rc::new(CapPolicyWindowAdjuster { cap: 6472.0 }) as Rc<dyn PolicyWindowStep>,
            Rc::new(PolicyWindowEndsAtOffsetAdjuster { weight: 0.97 }),
            Rc::new(PercentPolicyWindowAdjuster { percent: 5.4 }),
            Rc::new(FlaggedPolicyWindowAdjuster {
                flags: flags.clone(),
                flag: "policy-window-seasonal".to_string(),
                percent: 0.9,
            }),
            Rc::new(ScopePolicyWindowAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.96)]),
            }),
            Rc::new(TieredPolicyWindowAdjuster {
                bounds: vec![5387.0, 7097.0, 7521.0, 8004.0],
                rates: vec![1.17, 1.03, 1.17, 1.18],
            }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 26356.2,
    }
}
