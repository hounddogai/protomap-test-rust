use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the primary trust domain allocation calculation.
pub trait PrimaryTrustDomainAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryTrustDomainAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTrustDomainAllocationEndsAtOffsetAdjuster {
    pub weight: f64,
}

impl PrimaryTrustDomainAllocationFactor for PrimaryTrustDomainAllocationEndsAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ends at times {}", self.weight)
    }

    fn adjust(&self, item: &PrimaryTrustDomainAllocation, value: f64) -> f64 {
        value + (item.ends_at as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryTrustDomainAllocationAdjuster {
    pub minimum: f64,
}

impl PrimaryTrustDomainAllocationFactor for MinimumPrimaryTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryTrustDomainAllocation, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryTrustDomainAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryTrustDomainAllocationFactor for FlaggedPrimaryTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryTrustDomainAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryTrustDomainAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryTrustDomainAllocationFactor for TieredPrimaryTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryTrustDomainAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourcePrimaryTrustDomainAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryTrustDomainAllocationFactor for SourcePrimaryTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &PrimaryTrustDomainAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryTrustDomainAllocationAdjuster {
    pub percent: f64,
}

impl PrimaryTrustDomainAllocationFactor for PercentPrimaryTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryTrustDomainAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the primary trust domain allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryTrustDomainAllocationEngine {
    pub steps: Vec<Rc<dyn PrimaryTrustDomainAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryTrustDomainAllocationEngine {
    pub fn compute(&mut self, item: &PrimaryTrustDomainAllocation) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryTrustDomainAllocationFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryTrustDomainAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_primary_trust_domain_allocation_engine(flags: &flags::Registry) -> PrimaryTrustDomainAllocationEngine {
    PrimaryTrustDomainAllocationEngine {
        steps: vec![
            Rc::new(PrimaryTrustDomainAllocationEndsAtOffsetAdjuster { weight: 1.87 })
                as Rc<dyn PrimaryTrustDomainAllocationFactor>,
            Rc::new(MinimumPrimaryTrustDomainAllocationAdjuster { minimum: 0.3 }),
            Rc::new(FlaggedPrimaryTrustDomainAllocationAdjuster {
                flags: flags.clone(),
                flag: "primary-trust-domain-allocation-primary".to_string(),
                percent: -7.8,
            }),
            Rc::new(TieredPrimaryTrustDomainAllocationAdjuster {
                bounds: vec![483.0, 5274.0, 8555.0],
                rates: vec![0.98, 0.99, 0.93],
            }),
            Rc::new(SourcePrimaryTrustDomainAllocationAdjuster {
                factors: BTreeMap::from([("central".to_string(), 0.94), ("east".to_string(), 0.96)]),
            }),
            Rc::new(PercentPrimaryTrustDomainAllocationAdjuster { percent: 4.9 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 11638.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_trust_domain_allocation_engine(&flags::new_registry());
        let item = new_primary_trust_domain_allocation("code-21", "name-799");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
