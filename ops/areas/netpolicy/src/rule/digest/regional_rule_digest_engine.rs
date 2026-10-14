use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional rule digest calculation.
pub trait RegionalRuleDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalRuleDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalRuleDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalRuleDigestModifier for TieredRegionalRuleDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalRuleDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRegionalRuleDigestAdjuster {
    pub cap: f64,
}

impl RegionalRuleDigestModifier for CapRegionalRuleDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RegionalRuleDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRegionalRuleDigestAdjuster {
    pub percent: f64,
}

impl RegionalRuleDigestModifier for PercentRegionalRuleDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RegionalRuleDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRegionalRuleDigestAdjuster {
    pub minimum: f64,
}

impl RegionalRuleDigestModifier for MinimumRegionalRuleDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RegionalRuleDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the regional rule digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalRuleDigestEngine {
    pub steps: Vec<Rc<dyn RegionalRuleDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalRuleDigestEngine {
    pub fn run(&mut self, item: &RegionalRuleDigest) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[RegionalRuleDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_regional_rule_digest_engine() -> RegionalRuleDigestEngine {
    RegionalRuleDigestEngine {
        steps: vec![
            Rc::new(TieredRegionalRuleDigestAdjuster {
                bounds: vec![2397.0, 8575.0],
                rates: vec![1.2, 1.08],
            }) as Rc<dyn RegionalRuleDigestModifier>,
            Rc::new(CapRegionalRuleDigestAdjuster { cap: 45763.0 }),
            Rc::new(PercentRegionalRuleDigestAdjuster { percent: -13.3 }),
            Rc::new(MinimumRegionalRuleDigestAdjuster { minimum: 19.8 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 9013.6,
    }
}
