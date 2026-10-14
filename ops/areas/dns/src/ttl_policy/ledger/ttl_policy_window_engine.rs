use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the ttl policy window calculation.
pub trait TtlPolicyWindowAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyWindow, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTtlPolicyWindowAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TtlPolicyWindowAdjuster for RegionTtlPolicyWindowAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TtlPolicyWindow, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyWindowPriorityOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyWindowAdjuster for TtlPolicyWindowPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyWindow, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyWindowAdjuster {
    pub percent: f64,
}

impl TtlPolicyWindowAdjuster for PercentTtlPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyWindow, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTtlPolicyWindowAdjuster {
    pub cap: f64,
}

impl TtlPolicyWindowAdjuster for CapTtlPolicyWindowAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TtlPolicyWindow, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the ttl policy window steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyWindowEngine {
    pub steps: Vec<Rc<dyn TtlPolicyWindowAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyWindowEngine {
    pub fn run(&mut self, item: &TtlPolicyWindow) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyWindowAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyWindow]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_ttl_policy_window_engine() -> TtlPolicyWindowEngine {
    TtlPolicyWindowEngine {
        steps: vec![
            Rc::new(RegionTtlPolicyWindowAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.34)]),
            }) as Rc<dyn TtlPolicyWindowAdjuster>,
            Rc::new(TtlPolicyWindowPriorityOffsetAdjuster { weight: 0.66 }),
            Rc::new(PercentTtlPolicyWindowAdjuster { percent: -22.7 }),
            Rc::new(CapTtlPolicyWindowAdjuster { cap: 11500.0 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 81375.0,
    }
}
