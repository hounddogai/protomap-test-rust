use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the plan profile calculation.
pub trait PlanProfileAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanProfile, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanProfileAdjuster {
    pub percent: f64,
}

impl PlanProfileAdjuster for PercentPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanProfile, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanProfileAdjuster for TieredPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfileRevisionOffsetAdjuster {
    pub weight: f64,
}

impl PlanProfileAdjuster for PlanProfileRevisionOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus revision times {}", self.weight)
    }

    fn adjust(&self, item: &PlanProfile, value: f64) -> f64 {
        value + (item.revision as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanProfileAdjuster {
    pub cap: f64,
}

impl PlanProfileAdjuster for CapPlanProfileAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanProfile, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the plan profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanProfileEngine {
    pub steps: Vec<Rc<dyn PlanProfileAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanProfileEngine {
    pub fn evaluate(&mut self, item: &PlanProfile) -> f64 {
        let mut current = item.weighted_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanProfileAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_plan_profile_engine() -> PlanProfileEngine {
    PlanProfileEngine {
        steps: vec![
            Rc::new(PercentPlanProfileAdjuster { percent: 16.4 }) as Rc<dyn PlanProfileAdjuster>,
            Rc::new(TieredPlanProfileAdjuster {
                bounds: vec![304.0, 2809.0, 5377.0],
                rates: vec![0.8, 0.97, 0.83],
            }),
            Rc::new(PlanProfileRevisionOffsetAdjuster { weight: 0.93 }),
            Rc::new(CapPlanProfileAdjuster { cap: 33072.0 }),
        ],
        trace: Vec::new(),
        floor: 2.0,
        ceiling: 68297.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_profile_engine();
        let item = new_plan_profile("code-194", "label-337");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
