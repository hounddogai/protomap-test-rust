use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback trust domain allocation calculation.
pub trait FallbackTrustDomainAllocationFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackTrustDomainAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFallbackTrustDomainAllocationAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackTrustDomainAllocationFactor for ClusterFallbackTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FallbackTrustDomainAllocation, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackTrustDomainAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackTrustDomainAllocationFactor for FlaggedFallbackTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackTrustDomainAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFallbackTrustDomainAllocationAdjuster {
    pub percent: f64,
}

impl FallbackTrustDomainAllocationFactor for PercentFallbackTrustDomainAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FallbackTrustDomainAllocation, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the fallback trust domain allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackTrustDomainAllocationEngine {
    pub steps: Vec<Rc<dyn FallbackTrustDomainAllocationFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackTrustDomainAllocationEngine {
    pub fn run(&mut self, item: &FallbackTrustDomainAllocation) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[FallbackTrustDomainAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_fallback_trust_domain_allocation_engine(flags: &flags::Registry) -> FallbackTrustDomainAllocationEngine {
    FallbackTrustDomainAllocationEngine {
        steps: vec![
            Rc::new(ClusterFallbackTrustDomainAllocationAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.19),
                    ("online".to_string(), 1.39),
                    ("central".to_string(), 1.0),
                ]),
            }) as Rc<dyn FallbackTrustDomainAllocationFactor>,
            Rc::new(FlaggedFallbackTrustDomainAllocationAdjuster {
                flags: flags.clone(),
                flag: "fallback-trust-domain-allocation-regional".to_string(),
                percent: -1.7,
            }),
            Rc::new(PercentFallbackTrustDomainAllocationAdjuster { percent: -11.1 }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 6655.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_trust_domain_allocation_engine(&flags::new_registry());
        let item = new_fallback_trust_domain_allocation("code-825", "name-245");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
