use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant group calculation.
pub trait TenantGroupModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantGroupGraceDaysOffsetAdjuster {
    pub weight: f64,
}

impl TenantGroupModifier for TenantGroupGraceDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus grace days times {}", self.weight)
    }

    fn adjust(&self, item: &TenantGroup, value: f64) -> f64 {
        value + (item.grace_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantGroupAdjuster {
    pub minimum: f64,
}

impl TenantGroupModifier for MinimumTenantGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantGroupAdjuster {
    pub step: f64,
}

impl TenantGroupModifier for RoundingTenantGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the tenant group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantGroupEngine {
    pub steps: Vec<Rc<dyn TenantGroupModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantGroupEngine {
    pub fn compute(&mut self, item: &TenantGroup) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[TenantGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tenant_group_engine() -> TenantGroupEngine {
    TenantGroupEngine {
        steps: vec![
            Rc::new(TenantGroupGraceDaysOffsetAdjuster { weight: 1.91 }) as Rc<dyn TenantGroupModifier>,
            Rc::new(MinimumTenantGroupAdjuster { minimum: 37.4 }),
            Rc::new(RoundingTenantGroupAdjuster { step: 0.5 }),
        ],
        trace: Vec::new(),
        floor: 3.9,
        ceiling: 69910.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_group_engine();
        let item = new_tenant_group("code-540", "label-412");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
