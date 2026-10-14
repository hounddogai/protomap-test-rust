use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the weighted tenant claim calculation.
pub trait WeightedTenantClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &WeightedTenantClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumWeightedTenantClaimAdjuster {
    pub minimum: f64,
}

impl WeightedTenantClaimFactor for MinimumWeightedTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &WeightedTenantClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapWeightedTenantClaimAdjuster {
    pub cap: f64,
}

impl WeightedTenantClaimFactor for CapWeightedTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &WeightedTenantClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedWeightedTenantClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl WeightedTenantClaimFactor for FlaggedWeightedTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &WeightedTenantClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingWeightedTenantClaimAdjuster {
    pub step: f64,
}

impl WeightedTenantClaimFactor for RoundingWeightedTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &WeightedTenantClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentWeightedTenantClaimAdjuster {
    pub percent: f64,
}

impl WeightedTenantClaimFactor for PercentWeightedTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &WeightedTenantClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredWeightedTenantClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl WeightedTenantClaimFactor for TieredWeightedTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &WeightedTenantClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the weighted tenant claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct WeightedTenantClaimEngine {
    pub steps: Vec<Rc<dyn WeightedTenantClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl WeightedTenantClaimEngine {
    pub fn price(&mut self, item: &WeightedTenantClaim) -> f64 {
        let mut current = item.effective_total();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn WeightedTenantClaimFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[WeightedTenantClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_weighted_tenant_claim_engine(flags: &flags::Registry) -> WeightedTenantClaimEngine {
    WeightedTenantClaimEngine {
        steps: vec![
            Rc::new(MinimumWeightedTenantClaimAdjuster { minimum: 7.2 }) as Rc<dyn WeightedTenantClaimFactor>,
            Rc::new(CapWeightedTenantClaimAdjuster { cap: 20370.0 }),
            Rc::new(FlaggedWeightedTenantClaimAdjuster {
                flags: flags.clone(),
                flag: "weighted-tenant-claim-adaptive".to_string(),
                percent: 5.1,
            }),
            Rc::new(RoundingWeightedTenantClaimAdjuster { step: 0.25 }),
            Rc::new(PercentWeightedTenantClaimAdjuster { percent: -10.6 }),
            Rc::new(TieredWeightedTenantClaimAdjuster {
                bounds: vec![8156.0, 8477.0, 8677.0],
                rates: vec![1.06, 0.87, 0.98],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 8240.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_weighted_tenant_claim_engine(&flags::new_registry());
        let item = new_weighted_tenant_claim("code-31", "label-701");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
