use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the fallback alert rule calculation.
pub trait FallbackAlertRuleAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackAlertRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackAlertRuleAdjuster {
    pub percent: f64,
}

impl FallbackAlertRuleAdjuster for PercentFallbackAlertRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackAlertRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackAlertRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackAlertRuleAdjuster for TieredFallbackAlertRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackAlertRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelFallbackAlertRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackAlertRuleAdjuster for ChannelFallbackAlertRuleAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &FallbackAlertRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the fallback alert rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackAlertRuleEngine {
    pub steps: Vec<Rc<dyn FallbackAlertRuleAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackAlertRuleEngine {
    pub fn compute(&mut self, item: &FallbackAlertRule) -> f64 {
        let mut current = item.weighted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackAlertRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_fallback_alert_rule_engine() -> FallbackAlertRuleEngine {
    FallbackAlertRuleEngine {
        steps: vec![
            Rc::new(PercentFallbackAlertRuleAdjuster { percent: 2.9 }) as Rc<dyn FallbackAlertRuleAdjuster>,
            Rc::new(TieredFallbackAlertRuleAdjuster {
                bounds: vec![3379.0, 6030.0, 8180.0],
                rates: vec![0.89, 1.19, 1.12],
            }),
            Rc::new(ChannelFallbackAlertRuleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.85)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 4063.5,
    }
}
