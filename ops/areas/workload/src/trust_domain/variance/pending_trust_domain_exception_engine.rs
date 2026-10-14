use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending trust domain exception calculation.
pub trait PendingTrustDomainExceptionAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingTrustDomainException, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingTrustDomainExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingTrustDomainExceptionAdjuster for FlaggedPendingTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingTrustDomainException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamePendingTrustDomainExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PendingTrustDomainExceptionAdjuster for NamePendingTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &PendingTrustDomainException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingTrustDomainExceptionAdjuster {
    pub cap: f64,
}

impl PendingTrustDomainExceptionAdjuster for CapPendingTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingTrustDomainException, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingTrustDomainExceptionAdjuster {
    pub step: f64,
}

impl PendingTrustDomainExceptionAdjuster for RoundingPendingTrustDomainExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingTrustDomainException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the pending trust domain exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingTrustDomainExceptionEngine {
    pub steps: Vec<Rc<dyn PendingTrustDomainExceptionAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingTrustDomainExceptionEngine {
    pub fn compute(&mut self, item: &PendingTrustDomainException) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingTrustDomainException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_trust_domain_exception_engine(flags: &flags::Registry) -> PendingTrustDomainExceptionEngine {
    PendingTrustDomainExceptionEngine {
        steps: vec![
            Rc::new(FlaggedPendingTrustDomainExceptionAdjuster {
                flags: flags.clone(),
                flag: "pending-trust-domain-exception-regional".to_string(),
                percent: 0.6,
            }) as Rc<dyn PendingTrustDomainExceptionAdjuster>,
            Rc::new(NamePendingTrustDomainExceptionAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.4),
                    ("retail".to_string(), 0.99),
                    ("west".to_string(), 1.15),
                ]),
            }),
            Rc::new(CapPendingTrustDomainExceptionAdjuster { cap: 47667.0 }),
            Rc::new(RoundingPendingTrustDomainExceptionAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 28952.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_pending_trust_domain_exception_engine(&flags::new_registry());
        let item = new_pending_trust_domain_exception("code-141", "name-917");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
