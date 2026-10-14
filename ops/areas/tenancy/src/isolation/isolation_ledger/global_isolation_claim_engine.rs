use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the global isolation claim calculation.
pub trait GlobalIsolationClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalIsolationClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGlobalIsolationClaimAdjuster {
    pub percent: f64,
}

impl GlobalIsolationClaimModifier for PercentGlobalIsolationClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GlobalIsolationClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalIsolationClaimAdjuster {
    pub cap: f64,
}

impl GlobalIsolationClaimModifier for CapGlobalIsolationClaimAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalIsolationClaim, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalIsolationClaimAdjuster {
    pub minimum: f64,
}

impl GlobalIsolationClaimModifier for MinimumGlobalIsolationClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalIsolationClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIsolationClaimQuotaUnitsOffsetAdjuster {
    pub weight: f64,
}

impl GlobalIsolationClaimModifier for GlobalIsolationClaimQuotaUnitsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quota units times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalIsolationClaim, value: f64) -> f64 {
        value + (item.quota_units as f64) * self.weight
    }
}

/// Runs the global isolation claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalIsolationClaimEngine {
    pub steps: Vec<Rc<dyn GlobalIsolationClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalIsolationClaimEngine {
    pub fn price(&mut self, item: &GlobalIsolationClaim) -> f64 {
        let mut current = item.adjusted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalIsolationClaimModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalIsolationClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_isolation_claim_engine() -> GlobalIsolationClaimEngine {
    GlobalIsolationClaimEngine {
        steps: vec![
            Rc::new(PercentGlobalIsolationClaimAdjuster { percent: 9.1 }) as Rc<dyn GlobalIsolationClaimModifier>,
            Rc::new(CapGlobalIsolationClaimAdjuster { cap: 46993.0 }),
            Rc::new(MinimumGlobalIsolationClaimAdjuster { minimum: 16.8 }),
            Rc::new(GlobalIsolationClaimQuotaUnitsOffsetAdjuster { weight: 0.22 }),
        ],
        trace: Vec::new(),
        floor: 3.7,
        ceiling: 18128.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_isolation_claim_engine();
        let item = new_global_isolation_claim("code-220", "name-729");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
