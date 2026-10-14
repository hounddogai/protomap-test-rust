use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the approval rollup calculation.
pub trait ApprovalRollupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ApprovalRollup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapApprovalRollupAdjuster {
    pub cap: f64,
}

impl ApprovalRollupModifier for CapApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ApprovalRollup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentApprovalRollupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ApprovalRollupModifier for SegmentApprovalRollupAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &ApprovalRollup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedApprovalRollupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ApprovalRollupModifier for FlaggedApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ApprovalRollup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalRollupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ApprovalRollupModifier for TieredApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ApprovalRollup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumApprovalRollupAdjuster {
    pub minimum: f64,
}

impl ApprovalRollupModifier for MinimumApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ApprovalRollup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentApprovalRollupAdjuster {
    pub percent: f64,
}

impl ApprovalRollupModifier for PercentApprovalRollupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ApprovalRollup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the approval rollup steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ApprovalRollupEngine {
    pub steps: Vec<Rc<dyn ApprovalRollupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ApprovalRollupEngine {
    pub fn run(&mut self, item: &ApprovalRollup) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ApprovalRollupModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ApprovalRollup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_approval_rollup_engine(flags: &flags::Registry) -> ApprovalRollupEngine {
    ApprovalRollupEngine {
        steps: vec![
            Rc::new(CapApprovalRollupAdjuster { cap: 23855.0 }) as Rc<dyn ApprovalRollupModifier>,
            Rc::new(SegmentApprovalRollupAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.76)]),
            }),
            Rc::new(FlaggedApprovalRollupAdjuster {
                flags: flags.clone(),
                flag: "approval-rollup-manual".to_string(),
                percent: -4.6,
            }),
            Rc::new(TieredApprovalRollupAdjuster {
                bounds: vec![2882.0, 7713.0],
                rates: vec![0.85, 1.18],
            }),
            Rc::new(MinimumApprovalRollupAdjuster { minimum: 38.6 }),
            Rc::new(PercentApprovalRollupAdjuster { percent: -16.2 }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 95634.4,
    }
}
