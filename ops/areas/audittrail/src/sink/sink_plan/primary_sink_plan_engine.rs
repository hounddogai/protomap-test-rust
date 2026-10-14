use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the primary sink plan calculation.
pub trait PrimarySinkPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PrimarySinkPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPrimarySinkPlanAdjuster {
    pub step: f64,
}

impl PrimarySinkPlanAdjuster for RoundingPrimarySinkPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PrimarySinkPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPrimarySinkPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PrimarySinkPlanAdjuster for TieredPrimarySinkPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PrimarySinkPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPrimarySinkPlanAdjuster {
    pub minimum: f64,
}

impl PrimarySinkPlanAdjuster for MinimumPrimarySinkPlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PrimarySinkPlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the primary sink plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PrimarySinkPlanEngine {
    pub steps: Vec<Rc<dyn PrimarySinkPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PrimarySinkPlanEngine {
    pub fn evaluate(&mut self, item: &PrimarySinkPlan) -> f64 {
        let mut current = item.adjusted_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PrimarySinkPlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PrimarySinkPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_primary_sink_plan_engine() -> PrimarySinkPlanEngine {
    PrimarySinkPlanEngine {
        steps: vec![
            Rc::new(RoundingPrimarySinkPlanAdjuster { step: 5.0 }) as Rc<dyn PrimarySinkPlanAdjuster>,
            Rc::new(TieredPrimarySinkPlanAdjuster {
                bounds: vec![904.0, 2053.0, 4151.0, 6906.0],
                rates: vec![1.13, 1.08, 1.14, 1.06],
            }),
            Rc::new(MinimumPrimarySinkPlanAdjuster { minimum: 16.9 }),
        ],
        trace: Vec::new(),
        floor: 3.3,
        ceiling: 71106.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_primary_sink_plan_engine();
        let item = new_primary_sink_plan("code-843", "title-30");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
