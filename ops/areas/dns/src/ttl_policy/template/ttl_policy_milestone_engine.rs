use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the ttl policy milestone calculation.
pub trait TtlPolicyMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TtlPolicyMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTtlPolicyMilestoneAdjuster {
    pub percent: f64,
}

impl TtlPolicyMilestoneAdjuster for PercentTtlPolicyMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TtlPolicyMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTtlPolicyMilestoneAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TtlPolicyMilestoneAdjuster for FlaggedTtlPolicyMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TtlPolicyMilestone, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMilestoneScoreOffsetAdjuster {
    pub weight: f64,
}

impl TtlPolicyMilestoneAdjuster for TtlPolicyMilestoneScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &TtlPolicyMilestone, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTtlPolicyMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TtlPolicyMilestoneAdjuster for TieredTtlPolicyMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TtlPolicyMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the ttl policy milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TtlPolicyMilestoneEngine {
    pub steps: Vec<Rc<dyn TtlPolicyMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TtlPolicyMilestoneEngine {
    pub fn price(&mut self, item: &TtlPolicyMilestone) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TtlPolicyMilestoneAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TtlPolicyMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_ttl_policy_milestone_engine(flags: &flags::Registry) -> TtlPolicyMilestoneEngine {
    TtlPolicyMilestoneEngine {
        steps: vec![
            Rc::new(PercentTtlPolicyMilestoneAdjuster { percent: 2.4 }) as Rc<dyn TtlPolicyMilestoneAdjuster>,
            Rc::new(FlaggedTtlPolicyMilestoneAdjuster {
                flags: flags.clone(),
                flag: "ttl-policy-milestone-batched".to_string(),
                percent: 5.7,
            }),
            Rc::new(TtlPolicyMilestoneScoreOffsetAdjuster { weight: 0.46 }),
            Rc::new(TieredTtlPolicyMilestoneAdjuster {
                bounds: vec![1757.0, 7160.0],
                rates: vec![0.92, 0.93],
            }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 63754.3,
    }
}
