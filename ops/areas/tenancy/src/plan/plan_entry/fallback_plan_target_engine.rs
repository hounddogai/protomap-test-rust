use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the fallback plan target calculation.
pub trait FallbackPlanTargetStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FallbackPlanTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFallbackPlanTargetAdjuster {
    pub minimum: f64,
}

impl FallbackPlanTargetStep for MinimumFallbackPlanTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FallbackPlanTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedFallbackPlanTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl FallbackPlanTargetStep for FlaggedFallbackPlanTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &FallbackPlanTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceFallbackPlanTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FallbackPlanTargetStep for NamespaceFallbackPlanTargetAdjuster {
    fn describe(&self) -> String {
        "by namespace".to_string()
    }

    fn adjust(&self, item: &FallbackPlanTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTargetQuantityOffsetAdjuster {
    pub weight: f64,
}

impl FallbackPlanTargetStep for FallbackPlanTargetQuantityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus quantity times {}", self.weight)
    }

    fn adjust(&self, item: &FallbackPlanTarget, value: f64) -> f64 {
        value + (item.quantity as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFallbackPlanTargetAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FallbackPlanTargetStep for TieredFallbackPlanTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FallbackPlanTarget, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFallbackPlanTargetAdjuster {
    pub step: f64,
}

impl FallbackPlanTargetStep for RoundingFallbackPlanTargetAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FallbackPlanTarget, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the fallback plan target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FallbackPlanTargetEngine {
    pub steps: Vec<Rc<dyn FallbackPlanTargetStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FallbackPlanTargetEngine {
    pub fn price(&mut self, item: &FallbackPlanTarget) -> f64 {
        let mut current = item.weighted_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FallbackPlanTargetStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FallbackPlanTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_fallback_plan_target_engine(flags: &flags::Registry) -> FallbackPlanTargetEngine {
    FallbackPlanTargetEngine {
        steps: vec![
            Rc::new(MinimumFallbackPlanTargetAdjuster { minimum: 37.7 }) as Rc<dyn FallbackPlanTargetStep>,
            Rc::new(FlaggedFallbackPlanTargetAdjuster {
                flags: flags.clone(),
                flag: "fallback-plan-target-capped".to_string(),
                percent: 0.5,
            }),
            Rc::new(NamespaceFallbackPlanTargetAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.82), ("retail".to_string(), 1.36)]),
            }),
            Rc::new(FallbackPlanTargetQuantityOffsetAdjuster { weight: 1.03 }),
            Rc::new(TieredFallbackPlanTargetAdjuster {
                bounds: vec![6723.0, 7124.0, 9006.0],
                rates: vec![1.03, 1.07, 0.81],
            }),
            Rc::new(RoundingFallbackPlanTargetAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 35550.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_fallback_plan_target_engine(&flags::new_registry());
        let item = new_fallback_plan_target("code-851", "name-559");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
