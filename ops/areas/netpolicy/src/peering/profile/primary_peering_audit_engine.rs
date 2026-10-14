use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary peering audit calculation.
pub trait PrimaryPeeringAuditAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryPeeringAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryPeeringAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryPeeringAuditAdjuster for TieredPrimaryPeeringAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryPeeringAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringAuditPortOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryPeeringAuditAdjuster for PrimaryPeeringAuditPortOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus port times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryPeeringAudit, value: f64) -> f64 {
        value + (item.port as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPrimaryPeeringAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryPeeringAuditAdjuster for ChannelPrimaryPeeringAuditAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &PrimaryPeeringAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary peering audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryPeeringAuditEngine {
    pub steps: Vec<Rc<dyn PrimaryPeeringAuditAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryPeeringAuditEngine {
    pub fn run(&mut self, item: &PrimaryPeeringAudit) -> f64 {
        let mut current = item.adjusted_value();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryPeeringAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_peering_audit_engine() -> PrimaryPeeringAuditEngine {
    PrimaryPeeringAuditEngine {
        steps: vec![
            Rc::new(TieredPrimaryPeeringAuditAdjuster {
                bounds: vec![4801.0, 8021.0, 8265.0, 9265.0],
                rates: vec![0.89, 0.94, 1.07, 0.89],
            }) as Rc<dyn PrimaryPeeringAuditAdjuster>,
            Rc::new(PrimaryPeeringAuditPortOffsetAdjuster { weight: 1.7 }),
            Rc::new(ChannelPrimaryPeeringAuditAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 1.06),
                    ("retail".to_string(), 0.95),
                    ("south".to_string(), 0.86),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 28175.8,
    }
}
