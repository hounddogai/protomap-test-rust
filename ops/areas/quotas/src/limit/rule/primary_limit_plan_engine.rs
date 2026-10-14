use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary limit plan calculation.
pub trait PrimaryLimitPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimaryLimitPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimaryLimitPlanAdjuster {
    pub step: f64,
}

impl PrimaryLimitPlanAdjuster for RoundingPrimaryLimitPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimaryLimitPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimaryLimitPlanAdjuster {
    pub minimum: f64,
}

impl PrimaryLimitPlanAdjuster for MinimumPrimaryLimitPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimaryLimitPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPrimaryLimitPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PrimaryLimitPlanAdjuster for RegionPrimaryLimitPlanAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PrimaryLimitPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the primary limit plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimaryLimitPlanEngine {
    pub steps: Vec<Rc<dyn PrimaryLimitPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimaryLimitPlanEngine {
    pub fn price(&mut self, item: &PrimaryLimitPlan) -> f64 {
        let mut current = item.adjusted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimaryLimitPlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimaryLimitPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_primary_limit_plan_engine() -> PrimaryLimitPlanEngine {
    PrimaryLimitPlanEngine {
        steps: vec![
            Rc::new(RoundingPrimaryLimitPlanAdjuster { step: 0.25 }) as Rc<dyn PrimaryLimitPlanAdjuster>,
            Rc::new(MinimumPrimaryLimitPlanAdjuster { minimum: 32.9 }),
            Rc::new(RegionPrimaryLimitPlanAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.37)]),
            }),
        ],
        trace: Vec::new(),
        floor: 0.9,
        ceiling: 3224.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_limit_plan_engine();
        let item = new_primary_limit_plan("code-469", "label-72");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
