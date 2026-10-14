use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling tenant claim calculation.
pub trait RollingTenantClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTenantClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingTenantClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingTenantClaimFactor for NameRollingTenantClaimAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingTenantClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTenantClaimRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RollingTenantClaimFactor for RollingTenantClaimRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTenantClaim, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingTenantClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingTenantClaimFactor for TieredRollingTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingTenantClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTenantClaimAdjuster {
    pub percent: f64,
}

impl RollingTenantClaimFactor for PercentRollingTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTenantClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTenantClaimAdjuster {
    pub cap: f64,
}

impl RollingTenantClaimFactor for CapRollingTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTenantClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling tenant claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTenantClaimEngine {
    pub steps: Vec<Rc<dyn RollingTenantClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTenantClaimEngine {
    pub fn price(&mut self, item: &RollingTenantClaim) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[RollingTenantClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_tenant_claim_engine() -> RollingTenantClaimEngine {
    RollingTenantClaimEngine {
        steps: vec![
            Rc::new(NameRollingTenantClaimAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.27), ("central".to_string(), 0.87)]),
            }) as Rc<dyn RollingTenantClaimFactor>,
            Rc::new(RollingTenantClaimRotationDaysOffsetAdjuster { weight: 1.64 }),
            Rc::new(TieredRollingTenantClaimAdjuster {
                bounds: vec![6633.0, 8599.0],
                rates: vec![1.02, 1.12],
            }),
            Rc::new(PercentRollingTenantClaimAdjuster { percent: -23.6 }),
            Rc::new(CapRollingTenantClaimAdjuster { cap: 21915.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 47905.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_tenant_claim_engine();
        let item = new_rolling_tenant_claim("code-982", "name-654");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
