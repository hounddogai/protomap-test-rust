use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the ttl policy entry calculation.
pub trait TtlPolicyEntryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyEntryAdjuster {
    pub cap: f64,
}

impl TtlPolicyEntryModifier for CapTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTtlPolicyEntryAdjuster {
    pub step: f64,
}

impl TtlPolicyEntryModifier for RoundingTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TtlPolicyEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTtlPolicyEntryAdjuster {
    pub minimum: f64,
}

impl TtlPolicyEntryModifier for MinimumTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TtlPolicyEntry, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyEntryWindowDaysOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyEntryModifier for TtlPolicyEntryWindowDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus window days times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyEntry, value: f64) -> f64 {
        value + (item.window_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyEntryAdjuster {
    pub percent: f64,
}

impl TtlPolicyEntryModifier for PercentTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryTtlPolicyEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyEntryModifier for CategoryTtlPolicyEntryAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &TtlPolicyEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the ttl policy entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyEntryEngine {
    pub steps: Vec<Rc<dyn TtlPolicyEntryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyEntryEngine {
    pub fn run(&mut self, item: &TtlPolicyEntry) -> f64 {
        let mut current = item.projected_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyEntryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_entry_engine() -> TtlPolicyEntryEngine {
    TtlPolicyEntryEngine {
        steps: vec![
            Rc::new(CapTtlPolicyEntryAdjuster { cap: 15629.0 }) as Rc<dyn TtlPolicyEntryModifier>,
            Rc::new(RoundingTtlPolicyEntryAdjuster { step: 1.0 }),
            Rc::new(MinimumTtlPolicyEntryAdjuster { minimum: 38.3 }),
            Rc::new(TtlPolicyEntryWindowDaysOffsetAdjuster { weight: 1.27 }),
            Rc::new(PercentTtlPolicyEntryAdjuster { percent: -6.7 }),
            Rc::new(CategoryTtlPolicyEntryAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.01)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 57439.3,
    }
}
