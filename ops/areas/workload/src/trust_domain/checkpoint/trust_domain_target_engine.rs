use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain target calculation.
pub trait TrustDomainTargetAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainTargetAdjuster {
    pub cap: f64,
}

impl TrustDomainTargetAdjuster for CapTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTrustDomainTargetAdjuster {
    pub percent: f64,
}

impl TrustDomainTargetAdjuster for PercentTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TrustDomainTarget, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainTargetAdjuster for FlaggedTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainTargetAdjuster {
    pub step: f64,
}

impl TrustDomainTargetAdjuster for RoundingTrustDomainTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTargetSequenceOffsetAdjuster {
    pub weight: f64,
}

impl TrustDomainTargetAdjuster for TrustDomainTargetSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &TrustDomainTarget, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the trust domain target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainTargetEngine {
    pub steps: Vec<Rc<dyn TrustDomainTargetAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainTargetEngine {
    pub fn price(&mut self, item: &TrustDomainTarget) -> f64 {
        let mut current = item.net_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TrustDomainTargetAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_trust_domain_target_engine(flags: &flags::Registry) -> TrustDomainTargetEngine {
    TrustDomainTargetEngine {
        steps: vec![
            Rc::new(CapTrustDomainTargetAdjuster { cap: 49808.0 }) as Rc<dyn TrustDomainTargetAdjuster>,
            Rc::new(PercentTrustDomainTargetAdjuster { percent: -22.6 }),
            Rc::new(FlaggedTrustDomainTargetAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-target-weighted".to_string(),
                percent: -0.2,
            }),
            Rc::new(RoundingTrustDomainTargetAdjuster { step: 5.0 }),
            Rc::new(TrustDomainTargetSequenceOffsetAdjuster { weight: 1.85 }),
        ],
        trace: Vec::new(),
        floor: 1.4,
        ceiling: 14045.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_target_engine(&flags::new_registry());
        let item = new_trust_domain_target("code-136", "title-84");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
