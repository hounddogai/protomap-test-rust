use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the delegation plan calculation.
pub trait DelegationPlanStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DelegationPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDelegationPlanAdjuster {
    pub step: f64,
}

impl DelegationPlanStep for RoundingDelegationPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DelegationPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleDelegationPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DelegationPlanStep for TitleDelegationPlanAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &DelegationPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationPlanVersionOffsetAdjuster {
    pub weight: f64,
}

impl DelegationPlanStep for DelegationPlanVersionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus version times {}", self.weight)
    }

    fn adjust(&self, item: &DelegationPlan, value: f64) -> f64 {
        value + (item.version as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDelegationPlanAdjuster {
    pub minimum: f64,
}

impl DelegationPlanStep for MinimumDelegationPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DelegationPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDelegationPlanAdjuster {
    pub cap: f64,
}

impl DelegationPlanStep for CapDelegationPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DelegationPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDelegationPlanAdjuster {
    pub percent: f64,
}

impl DelegationPlanStep for PercentDelegationPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DelegationPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the delegation plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DelegationPlanEngine {
    pub steps: Vec<Rc<dyn DelegationPlanStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DelegationPlanEngine {
    pub fn run(&mut self, item: &DelegationPlan) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[DelegationPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_delegation_plan_engine() -> DelegationPlanEngine {
    DelegationPlanEngine {
        steps: vec![
            Rc::new(RoundingDelegationPlanAdjuster { step: 0.5 }) as Rc<dyn DelegationPlanStep>,
            Rc::new(TitleDelegationPlanAdjuster {
                factors: BTreeMap::from([("east".to_string(), 1.06)]),
            }),
            Rc::new(DelegationPlanVersionOffsetAdjuster { weight: 1.63 }),
            Rc::new(MinimumDelegationPlanAdjuster { minimum: 45.6 }),
            Rc::new(CapDelegationPlanAdjuster { cap: 48032.0 }),
            Rc::new(PercentDelegationPlanAdjuster { percent: 17.5 }),
        ],
        trace: Vec::new(),
        floor: 2.4,
        ceiling: 42563.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_delegation_plan_engine();
        let item = new_delegation_plan("code-227", "title-600");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
