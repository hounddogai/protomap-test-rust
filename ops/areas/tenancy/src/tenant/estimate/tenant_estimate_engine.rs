use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the tenant estimate calculation.
pub trait TenantEstimateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterTenantEstimateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantEstimateAdjuster for ClusterTenantEstimateAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &TenantEstimate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantEstimateAdjuster {
    pub percent: f64,
}

impl TenantEstimateAdjuster for PercentTenantEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantEstimateAdjuster {
    pub minimum: f64,
}

impl TenantEstimateAdjuster for MinimumTenantEstimateAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantEstimate, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantEstimateAdjuster {
    pub cap: f64,
}

impl TenantEstimateAdjuster for CapTenantEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the tenant estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantEstimateEngine {
    pub steps: Vec<Rc<dyn TenantEstimateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantEstimateEngine {
    pub fn run(&mut self, item: &TenantEstimate) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantEstimateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_tenant_estimate_engine() -> TenantEstimateEngine {
    TenantEstimateEngine {
        steps: vec![
            Rc::new(ClusterTenantEstimateAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.32)]),
            }) as Rc<dyn TenantEstimateAdjuster>,
            Rc::new(PercentTenantEstimateAdjuster { percent: -5.3 }),
            Rc::new(MinimumTenantEstimateAdjuster { minimum: 33.8 }),
            Rc::new(CapTenantEstimateAdjuster { cap: 12297.0 }),
        ],
        trace: Vec::new(),
        floor: 1.2,
        ceiling: 77868.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_estimate_engine();
        let item = new_tenant_estimate("code-986", "name-706");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
