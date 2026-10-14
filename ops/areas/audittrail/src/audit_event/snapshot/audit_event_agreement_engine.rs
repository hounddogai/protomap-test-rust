use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the audit event agreement calculation.
pub trait AuditEventAgreementFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventAgreement, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventAgreementAdjuster {
    pub step: f64,
}

impl AuditEventAgreementFactor for RoundingAuditEventAgreementAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventAgreement, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventAgreementAdjuster {
    pub cap: f64,
}

impl AuditEventAgreementFactor for CapAuditEventAgreementAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventAgreement, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventAgreementAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventAgreementFactor for TieredAuditEventAgreementAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventAgreement, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventAgreementAdjuster {
    pub minimum: f64,
}

impl AuditEventAgreementFactor for MinimumAuditEventAgreementAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventAgreement, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the audit event agreement steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventAgreementEngine {
    pub steps: Vec<Rc<dyn AuditEventAgreementFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventAgreementEngine {
    pub fn price(&mut self, item: &AuditEventAgreement) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventAgreementFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventAgreement]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_audit_event_agreement_engine() -> AuditEventAgreementEngine {
    AuditEventAgreementEngine {
        steps: vec![
            Rc::new(RoundingAuditEventAgreementAdjuster { step: 5.0 }) as Rc<dyn AuditEventAgreementFactor>,
            Rc::new(CapAuditEventAgreementAdjuster { cap: 49994.0 }),
            Rc::new(TieredAuditEventAgreementAdjuster {
                bounds: vec![6529.0, 6923.0],
                rates: vec![1.09, 0.83],
            }),
            Rc::new(MinimumAuditEventAgreementAdjuster { minimum: 29.3 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 70949.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_agreement_engine();
        let item = new_audit_event_agreement("code-395", "title-292");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
