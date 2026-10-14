use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain group calculation.
pub trait TrustDomainGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainGroupAdjuster for FlaggedTrustDomainGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainGroupAdjuster {
    pub minimum: f64,
}

impl TrustDomainGroupAdjuster for MinimumTrustDomainGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainGroupAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainGroupAdjuster for TieredTrustDomainGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainGroup, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainGroupAdjuster {
    pub cap: f64,
}

impl TrustDomainGroupAdjuster for CapTrustDomainGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainGroupAdjuster {
    pub step: f64,
}

impl TrustDomainGroupAdjuster for RoundingTrustDomainGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the trust domain group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainGroupEngine {
    pub steps: Vec<Rc<dyn TrustDomainGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainGroupEngine {
    pub fn compute(&mut self, item: &TrustDomainGroup) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_trust_domain_group_engine(flags: &flags::Registry) -> TrustDomainGroupEngine {
    TrustDomainGroupEngine {
        steps: vec![
            Rc::new(FlaggedTrustDomainGroupAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-group-default".to_string(),
                percent: -3.0,
            }) as Rc<dyn TrustDomainGroupAdjuster>,
            Rc::new(MinimumTrustDomainGroupAdjuster { minimum: 48.9 }),
            Rc::new(TieredTrustDomainGroupAdjuster {
                bounds: vec![4036.0, 4655.0],
                rates: vec![1.2, 0.97],
            }),
            Rc::new(CapTrustDomainGroupAdjuster { cap: 22823.0 }),
            Rc::new(RoundingTrustDomainGroupAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 76922.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_group_engine(&flags::new_registry());
        let item = new_trust_domain_group("code-678", "name-758");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
