use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the weighted rule quota calculation.
pub trait WeightedRuleQuotaFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedRuleQuota, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleQuotaBudgetOffsetAdjuster {
    pub weight: f64,
}

impl WeightedRuleQuotaFactor for WeightedRuleQuotaBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &WeightedRuleQuota, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedRuleQuotaAdjuster {
    pub step: f64,
}

impl WeightedRuleQuotaFactor for RoundingWeightedRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedRuleQuota, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedRuleQuotaAdjuster {
    pub percent: f64,
}

impl WeightedRuleQuotaFactor for PercentWeightedRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedRuleQuota, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedRuleQuotaAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedRuleQuotaFactor for TieredWeightedRuleQuotaAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedRuleQuota, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted rule quota steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedRuleQuotaEngine {
    pub steps: Vec<Rc<dyn WeightedRuleQuotaFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedRuleQuotaEngine {
    pub fn price(&mut self, item: &WeightedRuleQuota) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[WeightedRuleQuota]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_rule_quota_engine() -> WeightedRuleQuotaEngine {
    WeightedRuleQuotaEngine {
        steps: vec![
            Rc::new(WeightedRuleQuotaBudgetOffsetAdjuster { weight: 1.98 }) as Rc<dyn WeightedRuleQuotaFactor>,
            Rc::new(RoundingWeightedRuleQuotaAdjuster { step: 0.25 }),
            Rc::new(PercentWeightedRuleQuotaAdjuster { percent: 22.2 }),
            Rc::new(TieredWeightedRuleQuotaAdjuster {
                bounds: vec![5782.0, 6982.0],
                rates: vec![0.99, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 8752.3,
    }
}
