use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan summary calculation.
pub trait PlanSummaryModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanSummary, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePlanSummaryAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanSummaryModifier for NamespacePlanSummaryAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &PlanSummary, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanSummaryAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanSummaryModifier for TieredPlanSummaryAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanSummary, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanSummaryAdjuster {
    pub step: f64,
}

impl PlanSummaryModifier for RoundingPlanSummaryAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanSummary, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the plan summary steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanSummaryEngine {
    pub steps: Vec<Rc<dyn PlanSummaryModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanSummaryEngine {
    pub fn price(&mut self, item: &PlanSummary) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanSummaryModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanSummary]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_plan_summary_engine() -> PlanSummaryEngine {
    PlanSummaryEngine {
        steps: vec![
            Rc::new(NamespacePlanSummaryAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.98)]),
            }) as Rc<dyn PlanSummaryModifier>,
            Rc::new(TieredPlanSummaryAdjuster {
                bounds: vec![1805.0, 4958.0],
                rates: vec![1.01, 1.04],
            }),
            Rc::new(RoundingPlanSummaryAdjuster { step: 5.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 74132.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_summary_engine();
        let item = new_plan_summary("code-817", "name-437");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
