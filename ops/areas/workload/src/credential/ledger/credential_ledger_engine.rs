use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential ledger calculation.
pub trait CredentialLedgerStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CredentialLedgerStep for TieredCredentialLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CredentialLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceCredentialLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialLedgerStep for SourceCredentialLedgerAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &CredentialLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialLedgerQuantityOffsetAdjuster {
    pub weight: f64,
}

impl CredentialLedgerStep for CredentialLedgerQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialLedger, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialLedgerAdjuster {
    pub percent: f64,
}

impl CredentialLedgerStep for PercentCredentialLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the credential ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialLedgerEngine {
    pub steps: Vec<Rc<dyn CredentialLedgerStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialLedgerEngine {
    pub fn compute(&mut self, item: &CredentialLedger) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[CredentialLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_credential_ledger_engine() -> CredentialLedgerEngine {
    CredentialLedgerEngine {
        steps: vec![
            Rc::new(TieredCredentialLedgerAdjuster {
                bounds: vec![3200.0, 9335.0],
                rates: vec![0.96, 1.11],
            }) as Rc<dyn CredentialLedgerStep>,
            Rc::new(SourceCredentialLedgerAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 0.76),
                    ("retail".to_string(), 1.17),
                    ("south".to_string(), 1.31),
                ]),
            }),
            Rc::new(CredentialLedgerQuantityOffsetAdjuster { weight: 1.83 }),
            Rc::new(PercentCredentialLedgerAdjuster { percent: -5.6 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 73130.1,
    }
}
