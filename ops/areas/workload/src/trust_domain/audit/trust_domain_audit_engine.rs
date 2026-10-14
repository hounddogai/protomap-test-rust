use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain audit calculation.
pub trait TrustDomainAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainAuditAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainAuditStep for TieredTrustDomainAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainAudit, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainAuditAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainAuditStep for FlaggedTrustDomainAuditAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainAudit, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainAuditFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainAuditStep for TrustDomainAuditFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainAudit, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceTrustDomainAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainAuditStep for NamespaceTrustDomainAuditAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &TrustDomainAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainAuditAdjuster {
    pub percent: f64,
}

impl TrustDomainAuditStep for PercentTrustDomainAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainAuditEngine {
    pub steps: Vec<Rc<dyn TrustDomainAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainAuditEngine {
    pub fn compute(&mut self, item: &TrustDomainAudit) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_audit_engine(flags: &flags::Registry) -> TrustDomainAuditEngine {
    TrustDomainAuditEngine {
        steps: vec![
            Rc::new(TieredTrustDomainAuditAdjuster {
                bounds: vec![2417.0, 4640.0, 5461.0, 7364.0],
                rates: vec![1.02, 0.95, 1.04, 1.09],
            }) as Rc<dyn TrustDomainAuditStep>,
            Rc::new(FlaggedTrustDomainAuditAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-audit-lenient".to_string(),
                percent: -1.6,
            }),
            Rc::new(TrustDomainAuditFloorValueOffsetAdjuster { weight: 1.6 }),
            Rc::new(NamespaceTrustDomainAuditAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 0.85),
                    ("west".to_string(), 1.25),
                    ("online".to_string(), 1.35),
                ]),
            }),
            Rc::new(PercentTrustDomainAuditAdjuster { percent: -0.5 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 24912.4,
    }
}
