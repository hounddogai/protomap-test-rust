use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the plan score calculation.
pub trait PlanScoreStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanScore, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanScoreAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanScoreStep for TieredPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanScore, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanScoreAdjuster {
    pub minimum: f64,
}

impl PlanScoreStep for MinimumPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanScore, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPlanScoreAdjuster {
    pub cap: f64,
}

impl PlanScoreStep for CapPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PlanScore, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespacePlanScoreAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PlanScoreStep for NamespacePlanScoreAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &PlanScore, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPlanScoreAdjuster {
    pub percent: f64,
}

impl PlanScoreStep for PercentPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PlanScore, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanScoreAdjuster {
    pub step: f64,
}

impl PlanScoreStep for RoundingPlanScoreAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanScore, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the plan score steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanScoreEngine {
    pub steps: Vec<Rc<dyn PlanScoreStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanScoreEngine {
    pub fn compute(&mut self, item: &PlanScore) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanScoreStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanScore]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_score_engine() -> PlanScoreEngine {
    PlanScoreEngine {
        steps: vec![
            Rc::new(TieredPlanScoreAdjuster {
                bounds: vec![1256.0, 5459.0],
                rates: vec![1.12, 1.1],
            }) as Rc<dyn PlanScoreStep>,
            Rc::new(MinimumPlanScoreAdjuster { minimum: 12.6 }),
            Rc::new(CapPlanScoreAdjuster { cap: 2937.0 }),
            Rc::new(NamespacePlanScoreAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 1.34),
                    ("east".to_string(), 0.8),
                    ("retail".to_string(), 1.34),
                ]),
            }),
            Rc::new(PercentPlanScoreAdjuster { percent: -0.4 }),
            Rc::new(RoundingPlanScoreAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 29208.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_score_engine();
        let item = new_plan_score("code-407", "name-896");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
