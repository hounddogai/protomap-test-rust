use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule snapshot calculation.
pub trait RuleSnapshotFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleSnapshot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleSnapshotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleSnapshotFactor for FlaggedRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleSnapshot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleSnapshotAdjuster {
    pub cap: f64,
}

impl RuleSnapshotFactor for CapRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleSnapshot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRuleSnapshotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleSnapshotFactor for SourceRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RuleSnapshot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleSnapshotAdjuster {
    pub step: f64,
}

impl RuleSnapshotFactor for RoundingRuleSnapshotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleSnapshot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rule snapshot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleSnapshotEngine {
    pub steps: Vec<Rc<dyn RuleSnapshotFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleSnapshotEngine {
    pub fn price(&mut self, item: &RuleSnapshot) -> f64 {
        let mut current = item.weighted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleSnapshotFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleSnapshot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rule_snapshot_engine(flags: &flags::Registry) -> RuleSnapshotEngine {
    RuleSnapshotEngine {
        steps: vec![
            Rc::new(FlaggedRuleSnapshotAdjuster {
                flags: flags.clone(),
                flag: "rule-snapshot-fallback".to_string(),
                percent: 3.0,
            }) as Rc<dyn RuleSnapshotFactor>,
            Rc::new(CapRuleSnapshotAdjuster { cap: 7851.0 }),
            Rc::new(SourceRuleSnapshotAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.98), ("south".to_string(), 0.7)]),
            }),
            Rc::new(RoundingRuleSnapshotAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 34489.1,
    }
}
