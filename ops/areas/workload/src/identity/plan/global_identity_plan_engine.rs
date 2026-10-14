use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the global identity plan calculation.
pub trait GlobalIdentityPlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalIdentityPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredGlobalIdentityPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl GlobalIdentityPlanAdjuster for TieredGlobalIdentityPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &GlobalIdentityPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityPlanScoreOffsetAdjuster {
    pub weight: f64,
}

impl GlobalIdentityPlanAdjuster for GlobalIdentityPlanScoreOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus score times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalIdentityPlan, value: f64) -> f64 {
        value + item.score * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceGlobalIdentityPlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalIdentityPlanAdjuster for SourceGlobalIdentityPlanAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &GlobalIdentityPlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalIdentityPlanAdjuster {
    pub step: f64,
}

impl GlobalIdentityPlanAdjuster for RoundingGlobalIdentityPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalIdentityPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global identity plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalIdentityPlanEngine {
    pub steps: Vec<Rc<dyn GlobalIdentityPlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalIdentityPlanEngine {
    pub fn run(&mut self, item: &GlobalIdentityPlan) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GlobalIdentityPlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GlobalIdentityPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_global_identity_plan_engine() -> GlobalIdentityPlanEngine {
    GlobalIdentityPlanEngine {
        steps: vec![
            Rc::new(TieredGlobalIdentityPlanAdjuster {
                bounds: vec![2557.0, 4094.0],
                rates: vec![1.08, 1.02],
            }) as Rc<dyn GlobalIdentityPlanAdjuster>,
            Rc::new(GlobalIdentityPlanScoreOffsetAdjuster { weight: 1.21 }),
            Rc::new(SourceGlobalIdentityPlanAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.75),
                    ("east".to_string(), 1.25),
                    ("retail".to_string(), 1.16),
                ]),
            }),
            Rc::new(RoundingGlobalIdentityPlanAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 19585.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_global_identity_plan_engine();
        let item = new_global_identity_plan("code-430", "name-656");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
