use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule quota calculation.
pub trait RuleQuotaStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleQuotaAdjuster {
    pub minimum: f64,
}

impl RuleQuotaStep for MinimumRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleQuota, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleQuotaStep for TieredRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleQuotaAdjuster {
    pub cap: f64,
}

impl RuleQuotaStep for CapRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleQuota, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleQuotaAdjuster {
    pub step: f64,
}

impl RuleQuotaStep for RoundingRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleQuotaAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleQuotaStep for FlaggedRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleQuota, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryRuleQuotaAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleQuotaStep for CountryRuleQuotaAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &RuleQuota, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the rule quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleQuotaEngine {
    pub steps: Vec<Rc<dyn RuleQuotaStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleQuotaEngine {
    pub fn evaluate(&mut self, item: &RuleQuota) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RuleQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rule_quota_engine(flags: &flags::Registry) -> RuleQuotaEngine {
    RuleQuotaEngine {
        steps: vec![
            Rc::new(MinimumRuleQuotaAdjuster { minimum: 29.5 }) as Rc<dyn RuleQuotaStep>,
            Rc::new(TieredRuleQuotaAdjuster {
                bounds: vec![1299.0, 1503.0, 2423.0, 5647.0],
                rates: vec![0.85, 0.83, 0.9, 0.91],
            }),
            Rc::new(CapRuleQuotaAdjuster { cap: 42548.0 }),
            Rc::new(RoundingRuleQuotaAdjuster { step: 1.0 }),
            Rc::new(FlaggedRuleQuotaAdjuster {
                flags: flags.clone(),
                flag: "rule-quota-rolling".to_string(),
                percent: 8.9,
            }),
            Rc::new(CountryRuleQuotaAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 79659.9,
    }
}
