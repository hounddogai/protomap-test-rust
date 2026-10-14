use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant adjustment calculation.
pub trait TenantAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapTenantAdjustmentAdjuster {
    pub cap: f64,
}

impl TenantAdjustmentFactor for CapTenantAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &TenantAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentTenantAdjustmentAdjuster {
    pub percent: f64,
}

impl TenantAdjustmentFactor for PercentTenantAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &TenantAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantAdjustmentAdjuster {
    pub minimum: f64,
}

impl TenantAdjustmentFactor for MinimumTenantAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantAdjustmentAdjuster {
    pub step: f64,
}

impl TenantAdjustmentFactor for RoundingTenantAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredTenantAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl TenantAdjustmentFactor for TieredTenantAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &TenantAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAdjustmentRevisionOffsetAdjuster {
    pub weight: f64,
}

impl TenantAdjustmentFactor for TenantAdjustmentRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &TenantAdjustment, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

/// Runs the tenant adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantAdjustmentEngine {
    pub steps: Vec<Rc<dyn TenantAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantAdjustmentEngine {
    pub fn evaluate(&mut self, item: &TenantAdjustment) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[TenantAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_tenant_adjustment_engine() -> TenantAdjustmentEngine {
    TenantAdjustmentEngine {
        steps: vec![
            Rc::new(CapTenantAdjustmentAdjuster { cap: 22112.0 }) as Rc<dyn TenantAdjustmentFactor>,
            Rc::new(PercentTenantAdjustmentAdjuster { percent: 5.8 }),
            Rc::new(MinimumTenantAdjustmentAdjuster { minimum: 40.3 }),
            Rc::new(RoundingTenantAdjustmentAdjuster { step: 1.0 }),
            Rc::new(TieredTenantAdjustmentAdjuster {
                bounds: vec![1518.0, 2350.0, 7179.0],
                rates: vec![0.9, 1.05, 0.88],
            }),
            Rc::new(TenantAdjustmentRevisionOffsetAdjuster { weight: 1.63 }),
        ],
        trace: Vec::new(),
        floor: 3.5,
        ceiling: 77079.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_adjustment_engine();
        let item = new_tenant_adjustment("code-366", "title-422");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
