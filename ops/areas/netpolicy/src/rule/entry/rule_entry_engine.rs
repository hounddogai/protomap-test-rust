use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule entry calculation.
pub trait RuleEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleEntryRevisionOffsetAdjuster {
    pub weight: f64,
}

impl RuleEntryFactor for RuleEntryRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &RuleEntry, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelRuleEntryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleEntryFactor for LabelRuleEntryAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &RuleEntry, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleEntryFactor for FlaggedRuleEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleEntryAdjuster {
    pub step: f64,
}

impl RuleEntryFactor for RoundingRuleEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleEntryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleEntryFactor for TieredRuleEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleEntry, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rule entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleEntryEngine {
    pub steps: Vec<Rc<dyn RuleEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleEntryEngine {
    pub fn run(&mut self, item: &RuleEntry) -> f64 {
        let mut current = item.adjusted_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleEntryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rule_entry_engine(flags: &flags::Registry) -> RuleEntryEngine {
    RuleEntryEngine {
        steps: vec![
            Rc::new(RuleEntryRevisionOffsetAdjuster { weight: 1.25 }) as Rc<dyn RuleEntryFactor>,
            Rc::new(LabelRuleEntryAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 0.86), ("east".to_string(), 0.72)]),
            }),
            Rc::new(FlaggedRuleEntryAdjuster {
                flags: flags.clone(),
                flag: "rule-entry-regional".to_string(),
                percent: 2.3,
            }),
            Rc::new(RoundingRuleEntryAdjuster { step: 1.0 }),
            Rc::new(TieredRuleEntryAdjuster {
                bounds: vec![3030.0, 5199.0, 8897.0],
                rates: vec![1.04, 0.9, 1.15],
            }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 81647.9,
    }
}
