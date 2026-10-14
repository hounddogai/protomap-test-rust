use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the failover rule calculation.
pub trait FailoverRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FailoverRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFailoverRuleAdjuster {
    pub step: f64,
}

impl FailoverRuleModifier for RoundingFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FailoverRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleFailoverRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FailoverRuleModifier for LocaleFailoverRuleAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &FailoverRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FailoverRuleModifier for TieredFailoverRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FailoverRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the failover rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FailoverRuleEngine {
    pub steps: Vec<Rc<dyn FailoverRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FailoverRuleEngine {
    pub fn price(&mut self, item: &FailoverRule) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FailoverRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FailoverRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_failover_rule_engine() -> FailoverRuleEngine {
    FailoverRuleEngine {
        steps: vec![
            Rc::new(RoundingFailoverRuleAdjuster { step: 0.25 }) as Rc<dyn FailoverRuleModifier>,
            Rc::new(LocaleFailoverRuleAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.94),
                    ("retail".to_string(), 1.27),
                    ("north".to_string(), 0.94),
                ]),
            }),
            Rc::new(TieredFailoverRuleAdjuster {
                bounds: vec![7691.0, 8260.0],
                rates: vec![0.84, 0.92],
            }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 97515.4,
    }
}
