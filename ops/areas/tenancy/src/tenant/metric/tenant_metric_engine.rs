use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant metric calculation.
pub trait TenantMetricModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantMetric, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantMetricAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantMetricModifier for TieredTenantMetricAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantMetric, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantMetricAdjuster {
    pub step: f64,
}

impl TenantMetricModifier for RoundingTenantMetricAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantMetric, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopeTenantMetricAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl TenantMetricModifier for ScopeTenantMetricAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &TenantMetric, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the tenant metric steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantMetricEngine {
    pub steps: Vec<Rc<dyn TenantMetricModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantMetricEngine {
    pub fn compute(&mut self, item: &TenantMetric) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn TenantMetricModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[TenantMetric]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tenant_metric_engine() -> TenantMetricEngine {
    TenantMetricEngine {
        steps: vec![
            Rc::new(TieredTenantMetricAdjuster {
                bounds: vec![1072.0, 2898.0],
                rates: vec![0.85, 1.2],
            }) as Rc<dyn TenantMetricModifier>,
            Rc::new(RoundingTenantMetricAdjuster { step: 0.5 }),
            Rc::new(ScopeTenantMetricAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.87)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 43772.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_metric_engine();
        let item = new_tenant_metric("code-343", "label-838");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
