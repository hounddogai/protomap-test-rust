use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the primary identity claim calculation.
pub trait PrimaryIdentityClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryIdentityClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimaryIdentityClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimaryIdentityClaimAdjuster for TieredPrimaryIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimaryIdentityClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPrimaryIdentityClaimAdjuster {
    pub percent: f64,
}

impl PrimaryIdentityClaimAdjuster for PercentPrimaryIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PrimaryIdentityClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPrimaryIdentityClaimAdjuster {
    pub cap: f64,
}

impl PrimaryIdentityClaimAdjuster for CapPrimaryIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PrimaryIdentityClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPrimaryIdentityClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PrimaryIdentityClaimAdjuster for FlaggedPrimaryIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PrimaryIdentityClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryIdentityClaimAdjuster {
    pub step: f64,
}

impl PrimaryIdentityClaimAdjuster for RoundingPrimaryIdentityClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryIdentityClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the primary identity claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryIdentityClaimEngine {
    pub steps: Vec<Rc<dyn PrimaryIdentityClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryIdentityClaimEngine {
    pub fn run(&mut self, item: &PrimaryIdentityClaim) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryIdentityClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_primary_identity_claim_engine(flags: &flags::Registry) -> PrimaryIdentityClaimEngine {
    PrimaryIdentityClaimEngine {
        steps: vec![
            Rc::new(TieredPrimaryIdentityClaimAdjuster {
                bounds: vec![709.0, 2813.0, 3479.0, 5596.0],
                rates: vec![0.94, 1.12, 0.91, 1.2],
            }) as Rc<dyn PrimaryIdentityClaimAdjuster>,
            Rc::new(PercentPrimaryIdentityClaimAdjuster { percent: 12.8 }),
            Rc::new(CapPrimaryIdentityClaimAdjuster { cap: 34893.0 }),
            Rc::new(FlaggedPrimaryIdentityClaimAdjuster {
                flags: flags.clone(),
                flag: "primary-identity-claim-tiered".to_string(),
                percent: -3.2,
            }),
            Rc::new(RoundingPrimaryIdentityClaimAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 11495.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_identity_claim_engine(&flags::new_registry());
        let item = new_primary_identity_claim("code-751", "title-341");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
