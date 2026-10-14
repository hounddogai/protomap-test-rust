use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the release plan calculation.
pub trait ReleasePlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ReleasePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentReleasePlanAdjuster {
    pub percent: f64,
}

impl ReleasePlanAdjuster for PercentReleasePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ReleasePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumReleasePlanAdjuster {
    pub minimum: f64,
}

impl ReleasePlanAdjuster for MinimumReleasePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ReleasePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleasePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl ReleasePlanAdjuster for TieredReleasePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &ReleasePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the release plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ReleasePlanEngine {
    pub steps: Vec<Rc<dyn ReleasePlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ReleasePlanEngine {
    pub fn evaluate(&mut self, item: &ReleasePlan) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn ReleasePlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[ReleasePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_release_plan_engine() -> ReleasePlanEngine {
    ReleasePlanEngine {
        steps: vec![
            Rc::new(PercentReleasePlanAdjuster { percent: 18.2 }) as Rc<dyn ReleasePlanAdjuster>,
            Rc::new(MinimumReleasePlanAdjuster { minimum: 29.8 }),
            Rc::new(TieredReleasePlanAdjuster {
                bounds: vec![2465.0, 4889.0, 5184.0],
                rates: vec![0.92, 1.06, 0.94],
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 45515.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_release_plan_engine();
        let item = new_release_plan("code-14", "title-316");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
