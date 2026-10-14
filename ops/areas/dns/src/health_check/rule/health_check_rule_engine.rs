use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the health check rule calculation.
pub trait HealthCheckRuleFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &HealthCheckRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRuleThresholdOffsetAdjuster {
    pub weight: f64,
}

impl HealthCheckRuleFactor for HealthCheckRuleThresholdOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus threshold times {}", self.weight)
    }

    fn adjust(&self, item: &HealthCheckRule, value: f64) -> f64 {
        value + item.threshold * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedHealthCheckRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl HealthCheckRuleFactor for FlaggedHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &HealthCheckRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentHealthCheckRuleAdjuster {
    pub percent: f64,
}

impl HealthCheckRuleFactor for PercentHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &HealthCheckRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameHealthCheckRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl HealthCheckRuleFactor for NameHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &HealthCheckRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumHealthCheckRuleAdjuster {
    pub minimum: f64,
}

impl HealthCheckRuleFactor for MinimumHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &HealthCheckRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl HealthCheckRuleFactor for TieredHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &HealthCheckRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the health check rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct HealthCheckRuleEngine {
    pub steps: Vec<Rc<dyn HealthCheckRuleFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl HealthCheckRuleEngine {
    pub fn run(&mut self, item: &HealthCheckRule) -> f64 {
        let mut current = item.net_value();
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

    pub fn evaluate_all(&mut self, items: &[HealthCheckRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_health_check_rule_engine(flags: &flags::Registry) -> HealthCheckRuleEngine {
    HealthCheckRuleEngine {
        steps: vec![
            Rc::new(HealthCheckRuleThresholdOffsetAdjuster { weight: 1.82 }) as Rc<dyn HealthCheckRuleFactor>,
            Rc::new(FlaggedHealthCheckRuleAdjuster {
                flags: flags.clone(),
                flag: "health-check-rule-manual".to_string(),
                percent: -3.7,
            }),
            Rc::new(PercentHealthCheckRuleAdjuster { percent: 23.8 }),
            Rc::new(NameHealthCheckRuleAdjuster {
                factors: BTreeMap::from([("north".to_string(), 1.15)]),
            }),
            Rc::new(MinimumHealthCheckRuleAdjuster { minimum: 16.2 }),
            Rc::new(TieredHealthCheckRuleAdjuster {
                bounds: vec![4026.0, 8735.0, 9247.0],
                rates: vec![0.84, 1.14, 0.99],
            }),
        ],
        trace: Vec::new(),
        floor: 4.1,
        ceiling: 96824.3,
    }
}
