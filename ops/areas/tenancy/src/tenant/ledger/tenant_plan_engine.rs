use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant plan calculation.
pub trait TenantPlanFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantPlanFactor for TieredTenantPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantPlanAdjuster {
    pub minimum: f64,
}

impl TenantPlanFactor for MinimumTenantPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantPlanAdjuster {
    pub percent: f64,
}

impl TenantPlanFactor for PercentTenantPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the tenant plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantPlanEngine {
    pub steps: Vec<Rc<dyn TenantPlanFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantPlanEngine {
    pub fn evaluate(&mut self, item: &TenantPlan) -> f64 {
        let mut current = item.effective_total();
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

    pub fn evaluate_all(&mut self, items: &[TenantPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tenant_plan_engine() -> TenantPlanEngine {
    TenantPlanEngine {
        steps: vec![
            Rc::new(TieredTenantPlanAdjuster {
                bounds: vec![896.0, 3507.0, 9290.0],
                rates: vec![0.81, 1.15, 1.05],
            }) as Rc<dyn TenantPlanFactor>,
            Rc::new(MinimumTenantPlanAdjuster { minimum: 36.3 }),
            Rc::new(PercentTenantPlanAdjuster { percent: -22.2 }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 84654.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_plan_engine();
        let item = new_tenant_plan("code-254", "label-845");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
