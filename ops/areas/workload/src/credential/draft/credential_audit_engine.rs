use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the credential audit calculation.
pub trait CredentialAuditStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CredentialAudit, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCredentialAuditAdjuster {
    pub minimum: f64,
}

impl CredentialAuditStep for MinimumCredentialAuditAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CredentialAudit, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleCredentialAuditAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl CredentialAuditStep for LocaleCredentialAuditAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &CredentialAudit, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCredentialAuditAdjuster {
    pub step: f64,
}

impl CredentialAuditStep for RoundingCredentialAuditAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CredentialAudit, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCredentialAuditAdjuster {
    pub percent: f64,
}

impl CredentialAuditStep for PercentCredentialAuditAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CredentialAudit, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAuditAmountOffsetAdjuster {
    pub weight: f64,
}

impl CredentialAuditStep for CredentialAuditAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &CredentialAudit, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the credential audit steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CredentialAuditEngine {
    pub steps: Vec<Rc<dyn CredentialAuditStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CredentialAuditEngine {
    pub fn price(&mut self, item: &CredentialAudit) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[CredentialAudit]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_credential_audit_engine() -> CredentialAuditEngine {
    CredentialAuditEngine {
        steps: vec![
            Rc::new(MinimumCredentialAuditAdjuster { minimum: 36.5 }) as Rc<dyn CredentialAuditStep>,
            Rc::new(LocaleCredentialAuditAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.37), ("east".to_string(), 1.23)]),
            }),
            Rc::new(RoundingCredentialAuditAdjuster { step: 0.5 }),
            Rc::new(PercentCredentialAuditAdjuster { percent: -15.3 }),
            Rc::new(CredentialAuditAmountOffsetAdjuster { weight: 1.42 }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 37109.5,
    }
}
