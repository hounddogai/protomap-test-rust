use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule notice calculation.
pub trait RuleNoticeFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleNotice, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleNoticeAdjuster {
    pub minimum: f64,
}

impl RuleNoticeFactor for MinimumRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleNotice, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleNoticeAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleNoticeFactor for FlaggedRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleNotice, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleNoticeAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleNoticeFactor for TieredRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleNotice, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleNoticeAdjuster {
    pub step: f64,
}

impl RuleNoticeFactor for RoundingRuleNoticeAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleNotice, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rule notice steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleNoticeEngine {
    pub steps: Vec<Rc<dyn RuleNoticeFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleNoticeEngine {
    pub fn price(&mut self, item: &RuleNotice) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleNoticeFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleNotice]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rule_notice_engine(flags: &flags::Registry) -> RuleNoticeEngine {
    RuleNoticeEngine {
        steps: vec![
            Rc::new(MinimumRuleNoticeAdjuster { minimum: 43.0 }) as Rc<dyn RuleNoticeFactor>,
            Rc::new(FlaggedRuleNoticeAdjuster {
                flags: flags.clone(),
                flag: "rule-notice-primary".to_string(),
                percent: -5.6,
            }),
            Rc::new(TieredRuleNoticeAdjuster {
                bounds: vec![5515.0, 6397.0, 7077.0, 9354.0],
                rates: vec![1.02, 1.11, 0.99, 0.94],
            }),
            Rc::new(RoundingRuleNoticeAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 27609.1,
    }
}
