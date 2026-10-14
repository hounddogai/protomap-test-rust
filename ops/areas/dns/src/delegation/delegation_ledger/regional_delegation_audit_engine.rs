use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the regional delegation audit calculation.
pub trait RegionalDelegationAuditFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RegionalDelegationAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalDelegationAuditCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl RegionalDelegationAuditFactor for RegionalDelegationAuditCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &RegionalDelegationAudit, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRegionalDelegationAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RegionalDelegationAuditFactor for TieredRegionalDelegationAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RegionalDelegationAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelRegionalDelegationAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RegionalDelegationAuditFactor for ChannelRegionalDelegationAuditAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &RegionalDelegationAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the regional delegation audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RegionalDelegationAuditEngine {
    pub steps: Vec<Rc<dyn RegionalDelegationAuditFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RegionalDelegationAuditEngine {
    pub fn evaluate(&mut self, item: &RegionalDelegationAudit) -> f64 {
        let mut current = item.net_exposure();
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

    pub fn evaluate_all(&mut self, items: &[RegionalDelegationAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_regional_delegation_audit_engine() -> RegionalDelegationAuditEngine {
    RegionalDelegationAuditEngine {
        steps: vec![
            Rc::new(RegionalDelegationAuditCreatedAtOffsetAdjuster { weight: 1.02 })
                as Rc<dyn RegionalDelegationAuditFactor>,
            Rc::new(TieredRegionalDelegationAuditAdjuster {
                bounds: vec![2157.0, 9022.0, 9437.0],
                rates: vec![0.98, 0.92, 1.14],
            }),
            Rc::new(ChannelRegionalDelegationAuditAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.33),
                    ("north".to_string(), 1.25),
                    ("online".to_string(), 1.06),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 21217.4,
    }
}
