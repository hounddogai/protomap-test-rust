use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the seasonal isolation rule calculation.
pub trait SeasonalIsolationRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalIsolationRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalIsolationRuleAdjuster {
    pub minimum: f64,
}

impl SeasonalIsolationRuleModifier for MinimumSeasonalIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalIsolationRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredSeasonalIsolationRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl SeasonalIsolationRuleModifier for TieredSeasonalIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &SeasonalIsolationRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameSeasonalIsolationRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalIsolationRuleModifier for NameSeasonalIsolationRuleAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &SeasonalIsolationRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalIsolationRuleAdjuster {
    pub step: f64,
}

impl SeasonalIsolationRuleModifier for RoundingSeasonalIsolationRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalIsolationRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal isolation rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalIsolationRuleEngine {
    pub steps: Vec<Rc<dyn SeasonalIsolationRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalIsolationRuleEngine {
    pub fn compute(&mut self, item: &SeasonalIsolationRule) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn SeasonalIsolationRuleModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[SeasonalIsolationRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_seasonal_isolation_rule_engine() -> SeasonalIsolationRuleEngine {
    SeasonalIsolationRuleEngine {
        steps: vec![
            Rc::new(MinimumSeasonalIsolationRuleAdjuster { minimum: 18.7 }) as Rc<dyn SeasonalIsolationRuleModifier>,
            Rc::new(TieredSeasonalIsolationRuleAdjuster {
                bounds: vec![2103.0, 2533.0, 5340.0],
                rates: vec![0.96, 1.18, 0.88],
            }),
            Rc::new(NameSeasonalIsolationRuleAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.28)]),
            }),
            Rc::new(RoundingSeasonalIsolationRuleAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 90170.8,
    }
}
