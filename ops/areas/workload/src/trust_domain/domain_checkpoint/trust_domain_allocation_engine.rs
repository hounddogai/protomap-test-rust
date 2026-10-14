use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the trust domain allocation calculation.
pub trait TrustDomainAllocationStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TrustDomainAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedTrustDomainAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl TrustDomainAllocationStep for FlaggedTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &TrustDomainAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTrustDomainAllocationAdjuster {
    pub cap: f64,
}

impl TrustDomainAllocationStep for CapTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TrustDomainAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTrustDomainAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TrustDomainAllocationStep for NameTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &TrustDomainAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTrustDomainAllocationAdjuster {
    pub step: f64,
}

impl TrustDomainAllocationStep for RoundingTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TrustDomainAllocation, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTrustDomainAllocationAdjuster {
    pub minimum: f64,
}

impl TrustDomainAllocationStep for MinimumTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TrustDomainAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTrustDomainAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TrustDomainAllocationStep for TieredTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TrustDomainAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the trust domain allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TrustDomainAllocationEngine {
    pub steps: Vec<Rc<dyn TrustDomainAllocationStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TrustDomainAllocationEngine {
    pub fn evaluate(&mut self, item: &TrustDomainAllocation) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TrustDomainAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_trust_domain_allocation_engine(flags: &flags::Registry) -> TrustDomainAllocationEngine {
    TrustDomainAllocationEngine {
        steps: vec![
            Rc::new(FlaggedTrustDomainAllocationAdjuster {
                flags: flags.clone(),
                flag: "trust-domain-allocation-default".to_string(),
                percent: 3.5,
            }) as Rc<dyn TrustDomainAllocationStep>,
            Rc::new(CapTrustDomainAllocationAdjuster { cap: 47034.0 }),
            Rc::new(NameTrustDomainAllocationAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.29),
                    ("west".to_string(), 1.1),
                    ("retail".to_string(), 0.89),
                ]),
            }),
            Rc::new(RoundingTrustDomainAllocationAdjuster { step: 0.05 }),
            Rc::new(MinimumTrustDomainAllocationAdjuster { minimum: 21.3 }),
            Rc::new(TieredTrustDomainAllocationAdjuster {
                bounds: vec![5017.0, 6345.0, 7712.0],
                rates: vec![1.11, 1.02, 1.16],
            }),
        ],
        trace: Vec::new(),
        floor: 2.6,
        ceiling: 21569.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_trust_domain_allocation_engine(&flags::new_registry());
        let item = new_trust_domain_allocation("code-223", "name-45");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
