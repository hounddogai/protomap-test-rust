use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant claim calculation.
pub trait TenantClaimFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantClaimAdjuster {
    pub cap: f64,
}

impl TenantClaimFactor for CapTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionTenantClaimAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantClaimFactor for RegionTenantClaimAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &TenantClaim, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantClaimAdjuster {
    pub percent: f64,
}

impl TenantClaimFactor for PercentTenantClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantClaimSequenceOffsetAdjuster {
    pub weight: f64,
}

impl TenantClaimFactor for TenantClaimSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &TenantClaim, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

/// Runs the tenant claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantClaimEngine {
    pub steps: Vec<Rc<dyn TenantClaimFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantClaimEngine {
    pub fn price(&mut self, item: &TenantClaim) -> f64 {
        let mut current = item.adjusted_load();
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

    pub fn evaluate_all(&mut self, items: &[TenantClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_tenant_claim_engine() -> TenantClaimEngine {
    TenantClaimEngine {
        steps: vec![
            Rc::new(CapTenantClaimAdjuster { cap: 44128.0 }) as Rc<dyn TenantClaimFactor>,
            Rc::new(RegionTenantClaimAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.26)]),
            }),
            Rc::new(PercentTenantClaimAdjuster { percent: 24.2 }),
            Rc::new(TenantClaimSequenceOffsetAdjuster { weight: 0.65 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 37130.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_claim_engine();
        let item = new_tenant_claim("code-764", "title-167");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
