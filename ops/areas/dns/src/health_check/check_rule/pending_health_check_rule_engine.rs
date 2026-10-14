use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the pending health check rule calculation.
pub trait PendingHealthCheckRuleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingHealthCheckRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingHealthCheckRuleAdjuster {
    pub cap: f64,
}

impl PendingHealthCheckRuleModifier for CapPendingHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingHealthCheckRule, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingHealthCheckRuleSequenceOffsetAdjuster {
    pub weight: f64,
}

impl PendingHealthCheckRuleModifier for PendingHealthCheckRuleSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &PendingHealthCheckRule, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPendingHealthCheckRuleAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PendingHealthCheckRuleModifier for TieredPendingHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PendingHealthCheckRule, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPendingHealthCheckRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingHealthCheckRuleModifier for SegmentPendingHealthCheckRuleAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PendingHealthCheckRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the pending health check rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingHealthCheckRuleEngine {
    pub steps: Vec<Rc<dyn PendingHealthCheckRuleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingHealthCheckRuleEngine {
    pub fn evaluate(&mut self, item: &PendingHealthCheckRule) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[PendingHealthCheckRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_pending_health_check_rule_engine() -> PendingHealthCheckRuleEngine {
    PendingHealthCheckRuleEngine {
        steps: vec![
            Rc::new(CapPendingHealthCheckRuleAdjuster { cap: 2586.0 }) as Rc<dyn PendingHealthCheckRuleModifier>,
            Rc::new(PendingHealthCheckRuleSequenceOffsetAdjuster { weight: 1.48 }),
            Rc::new(TieredPendingHealthCheckRuleAdjuster {
                bounds: vec![3099.0, 9287.0, 9812.0],
                rates: vec![0.96, 1.02, 1.02],
            }),
            Rc::new(SegmentPendingHealthCheckRuleAdjuster {
                factors: BTreeMap::from([("east".to_string(), 0.89)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 67938.9,
    }
}
