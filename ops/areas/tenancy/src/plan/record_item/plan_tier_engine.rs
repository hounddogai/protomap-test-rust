use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the plan tier calculation.
pub trait PlanTierStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PlanTier, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPlanTierAdjuster {
    pub minimum: f64,
}

impl PlanTierStep for MinimumPlanTierAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PlanTier, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPlanTierAdjuster {
    pub step: f64,
}

impl PlanTierStep for RoundingPlanTierAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PlanTier, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPlanTierAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PlanTierStep for TieredPlanTierAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PlanTier, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPlanTierAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PlanTierStep for FlaggedPlanTierAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PlanTier, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the plan tier steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PlanTierEngine {
    pub steps: Vec<Rc<dyn PlanTierStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PlanTierEngine {
    pub fn compute(&mut self, item: &PlanTier) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PlanTierStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PlanTier]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_plan_tier_engine(flags: &flags::Registry) -> PlanTierEngine {
    PlanTierEngine {
        steps: vec![
            Rc::new(MinimumPlanTierAdjuster { minimum: 9.2 }) as Rc<dyn PlanTierStep>,
            Rc::new(RoundingPlanTierAdjuster { step: 1.0 }),
            Rc::new(TieredPlanTierAdjuster {
                bounds: vec![2964.0, 4240.0, 6111.0],
                rates: vec![1.16, 0.98, 0.92],
            }),
            Rc::new(FlaggedPlanTierAdjuster {
                flags: flags.clone(),
                flag: "plan-tier-default".to_string(),
                percent: 8.8,
            }),
        ],
        trace: Vec::new(),
        floor: 1.8,
        ceiling: 30896.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_plan_tier_engine(&flags::new_registry());
        let item = new_plan_tier("code-139", "label-466");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
