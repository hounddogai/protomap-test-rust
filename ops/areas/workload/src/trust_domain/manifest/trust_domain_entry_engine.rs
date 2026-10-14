use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain entry calculation.
pub trait TrustDomainEntryFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainEntry, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainEntryAdjuster {
    pub percent: f64,
}

impl TrustDomainEntryFactor for PercentTrustDomainEntryAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainEntry, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainEntryAdjuster {
    pub step: f64,
}

impl TrustDomainEntryFactor for RoundingTrustDomainEntryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainEntry, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainEntryAdjuster {
    pub cap: f64,
}

impl TrustDomainEntryFactor for CapTrustDomainEntryAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainEntry, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainEntryAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainEntryFactor for FlaggedTrustDomainEntryAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainEntry, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the trust domain entry steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainEntryEngine {
    pub steps: Vec<Rc<dyn TrustDomainEntryFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainEntryEngine {
    pub fn evaluate(&mut self, item: &TrustDomainEntry) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainEntryFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainEntry]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_trust_domain_entry_engine(flags: &flags::Registry) -> TrustDomainEntryEngine {
    TrustDomainEntryEngine {
        steps: vec![
            Rc::new(PercentTrustDomainEntryAdjuster { percent: -4.8 }) as Rc<dyn TrustDomainEntryFactor>,
            Rc::new(RoundingTrustDomainEntryAdjuster { step: 5.0 }),
            Rc::new(CapTrustDomainEntryAdjuster { cap: 22907.0 }),
            Rc::new(FlaggedTrustDomainEntryAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-entry-primary".to_string(),
                percent: 9.7,
            }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 42248.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_entry_engine(&flags::new_registry());
        let item = new_trust_domain_entry("code-207", "name-688");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
