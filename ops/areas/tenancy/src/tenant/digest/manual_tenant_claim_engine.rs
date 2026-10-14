use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the manual tenant claim calculation.
pub trait ManualTenantClaimAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ManualTenantClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumManualTenantClaimAdjuster {
    pub minimum: f64,
}

impl ManualTenantClaimAdjuster for MinimumManualTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ManualTenantClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredManualTenantClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ManualTenantClaimAdjuster for TieredManualTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ManualTenantClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentManualTenantClaimAdjuster {
    pub percent: f64,
}

impl ManualTenantClaimAdjuster for PercentManualTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ManualTenantClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedManualTenantClaimAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ManualTenantClaimAdjuster for FlaggedManualTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ManualTenantClaim, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantClaimGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl ManualTenantClaimAdjuster for ManualTenantClaimGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &ManualTenantClaim, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

/// Runs the manual tenant claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ManualTenantClaimEngine {
    pub steps: Vec<Rc<dyn ManualTenantClaimAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ManualTenantClaimEngine {
    pub fn price(&mut self, item: &ManualTenantClaim) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ManualTenantClaimAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ManualTenantClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_manual_tenant_claim_engine(flags: &flags::Registry) -> ManualTenantClaimEngine {
    ManualTenantClaimEngine {
        steps: vec![
            Rc::new(MinimumManualTenantClaimAdjuster { minimum: 41.8 }) as Rc<dyn ManualTenantClaimAdjuster>,
            Rc::new(TieredManualTenantClaimAdjuster {
                bounds: vec![87.0, 2005.0, 2750.0, 4281.0],
                rates: vec![0.82, 1.03, 1.11, 1.0],
            }),
            Rc::new(PercentManualTenantClaimAdjuster { percent: -19.1 }),
            Rc::new(FlaggedManualTenantClaimAdjuster {
                flags: flags.clone(),
                flag: "manual-tenant-claim-batched".to_string(),
                percent: -5.5,
            }),
            Rc::new(ManualTenantClaimGraceDaysOffsetAdjuster { weight: 0.16 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 8722.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_manual_tenant_claim_engine(&flags::new_registry());
        let item = new_manual_tenant_claim("code-709", "title-729");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
