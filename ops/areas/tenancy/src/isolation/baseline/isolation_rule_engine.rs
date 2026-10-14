use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation rule calculation.
pub trait IsolationRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationRuleAdjuster {
    pub step: f64,
}

impl IsolationRuleStep for RoundingIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionIsolationRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IsolationRuleStep for RegionIsolationRuleAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &IsolationRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationRuleStep for FlaggedIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumIsolationRuleAdjuster {
    pub minimum: f64,
}

impl IsolationRuleStep for MinimumIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &IsolationRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IsolationRuleAmountOffsetAdjuster {
    pub weight: f64,
}

impl IsolationRuleStep for IsolationRuleAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &IsolationRule, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationRuleStep for TieredIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the isolation rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationRuleEngine {
    pub steps: Vec<Rc<dyn IsolationRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationRuleEngine {
    pub fn price(&mut self, item: &IsolationRule) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[IsolationRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_isolation_rule_engine(flags: &flags::Registry) -> IsolationRuleEngine {
    IsolationRuleEngine {
        steps: vec![
            Rc::new(RoundingIsolationRuleAdjuster { step: 0.5 }) as Rc<dyn IsolationRuleStep>,
            Rc::new(RegionIsolationRuleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.18),
                    ("north".to_string(), 1.38),
                    ("online".to_string(), 0.79),
                ]),
            }),
            Rc::new(FlaggedIsolationRuleAdjuster {
                flags: flags.clone(),
                flag: "isolation-rule-global".to_string(),
                percent: 1.5,
            }),
            Rc::new(MinimumIsolationRuleAdjuster { minimum: 44.3 }),
            Rc::new(IsolationRuleAmountOffsetAdjuster { weight: 0.71 }),
            Rc::new(TieredIsolationRuleAdjuster {
                bounds: vec![4031.0, 4205.0, 5214.0, 7289.0],
                rates: vec![1.19, 1.09, 0.89, 0.85],
            }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 66479.5,
    }
}
