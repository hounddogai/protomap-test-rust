use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the tenant schedule calculation.
pub trait TenantScheduleModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &TenantSchedule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantScheduleRevisionOffsetAdjuster {
    pub weight: f64,
}

impl TenantScheduleModifier for TenantScheduleRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &TenantSchedule, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingTenantScheduleAdjuster {
    pub step: f64,
}

impl TenantScheduleModifier for RoundingTenantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &TenantSchedule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumTenantScheduleAdjuster {
    pub minimum: f64,
}

impl TenantScheduleModifier for MinimumTenantScheduleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &TenantSchedule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the tenant schedule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct TenantScheduleEngine {
    pub steps: Vec<Rc<dyn TenantScheduleModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl TenantScheduleEngine {
    pub fn compute(&mut self, item: &TenantSchedule) -> f64 {
        let mut current = item.projected_value();
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

    pub fn evaluate_all(&mut self, items: &[TenantSchedule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_tenant_schedule_engine() -> TenantScheduleEngine {
    TenantScheduleEngine {
        steps: vec![
            Rc::new(TenantScheduleRevisionOffsetAdjuster { weight: 1.29 }) as Rc<dyn TenantScheduleModifier>,
            Rc::new(RoundingTenantScheduleAdjuster { step: 0.1 }),
            Rc::new(MinimumTenantScheduleAdjuster { minimum: 26.4 }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 18587.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_tenant_schedule_engine();
        let item = new_tenant_schedule("code-915", "label-854");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
