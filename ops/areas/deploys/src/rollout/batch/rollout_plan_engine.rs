use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rollout plan calculation.
pub trait RolloutPlanModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RolloutPlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRolloutPlanAdjuster {
    pub percent: f64,
}

impl RolloutPlanModifier for PercentRolloutPlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RolloutPlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRolloutPlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RolloutPlanModifier for TieredRolloutPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RolloutPlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRolloutPlanAdjuster {
    pub cap: f64,
}

impl RolloutPlanModifier for CapRolloutPlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RolloutPlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRolloutPlanAdjuster {
    pub step: f64,
}

impl RolloutPlanModifier for RoundingRolloutPlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RolloutPlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPlanCeilingOffsetAdjuster {
    pub weight: f64,
}

impl RolloutPlanModifier for RolloutPlanCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &RolloutPlan, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRolloutPlanAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RolloutPlanModifier for FlaggedRolloutPlanAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RolloutPlan, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rollout plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RolloutPlanEngine {
    pub steps: Vec<Rc<dyn RolloutPlanModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RolloutPlanEngine {
    pub fn run(&mut self, item: &RolloutPlan) -> f64 {
        let mut current = item.weighted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RolloutPlanModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RolloutPlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rollout_plan_engine(flags: &flags::Registry) -> RolloutPlanEngine {
    RolloutPlanEngine {
        steps: vec![
            Rc::new(PercentRolloutPlanAdjuster { percent: 7.8 }) as Rc<dyn RolloutPlanModifier>,
            Rc::new(TieredRolloutPlanAdjuster {
                bounds: vec![4529.0, 7950.0, 8754.0],
                rates: vec![1.19, 0.83, 0.91],
            }),
            Rc::new(CapRolloutPlanAdjuster { cap: 44913.0 }),
            Rc::new(RoundingRolloutPlanAdjuster { step: 0.1 }),
            Rc::new(RolloutPlanCeilingOffsetAdjuster { weight: 2.0 }),
            Rc::new(FlaggedRolloutPlanAdjuster {
                flags: flags.clone(),
                flag: "rollout-plan-pending".to_string(),
                percent: 1.3,
            }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 97203.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rollout_plan_engine(&flags::new_registry());
        let item = new_rollout_plan("code-73", "title-900");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
