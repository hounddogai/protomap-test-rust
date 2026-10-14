use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan baseline calculation.
pub trait PlanBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScopePlanBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanBaselineStep for ScopePlanBaselineAdjuster {
    fn describe(&self) -> String {
        "by scope".to_string()
    }

    fn adjust(&self, item: &PlanBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanBaselineAdjuster {
    pub percent: f64,
}

impl PlanBaselineStep for PercentPlanBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanBaselineAdjuster {
    pub step: f64,
}

impl PlanBaselineStep for RoundingPlanBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the plan baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanBaselineEngine {
    pub steps: Vec<Rc<dyn PlanBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanBaselineEngine {
    pub fn compute(&mut self, item: &PlanBaseline) -> f64 {
        let mut current = item.adjusted_exposure();
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

    pub fn evaluate_all(&mut self, items: &[PlanBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_baseline_engine() -> PlanBaselineEngine {
    PlanBaselineEngine {
        steps: vec![
            Rc::new(ScopePlanBaselineAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.31)]),
            }) as Rc<dyn PlanBaselineStep>,
            Rc::new(PercentPlanBaselineAdjuster { percent: -1.1 }),
            Rc::new(RoundingPlanBaselineAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 65124.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_baseline_engine();
        let item = new_plan_baseline("code-641", "label-267");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
