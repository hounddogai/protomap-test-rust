use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the freeze plan calculation.
pub trait FreezePlanAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &FreezePlan, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingFreezePlanAdjuster {
    pub step: f64,
}

impl FreezePlanAdjuster for RoundingFreezePlanAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &FreezePlan, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezePlanAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl FreezePlanAdjuster for TieredFreezePlanAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &FreezePlan, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterFreezePlanAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl FreezePlanAdjuster for ClusterFreezePlanAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &FreezePlan, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentFreezePlanAdjuster {
    pub percent: f64,
}

impl FreezePlanAdjuster for PercentFreezePlanAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &FreezePlan, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumFreezePlanAdjuster {
    pub minimum: f64,
}

impl FreezePlanAdjuster for MinimumFreezePlanAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &FreezePlan, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapFreezePlanAdjuster {
    pub cap: f64,
}

impl FreezePlanAdjuster for CapFreezePlanAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &FreezePlan, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the freeze plan steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct FreezePlanEngine {
    pub steps: Vec<Rc<dyn FreezePlanAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl FreezePlanEngine {
    pub fn evaluate(&mut self, item: &FreezePlan) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn FreezePlanAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[FreezePlan]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_freeze_plan_engine() -> FreezePlanEngine {
    FreezePlanEngine {
        steps: vec![
            Rc::new(RoundingFreezePlanAdjuster { step: 5.0 }) as Rc<dyn FreezePlanAdjuster>,
            Rc::new(TieredFreezePlanAdjuster {
                bounds: vec![2948.0, 6710.0, 8642.0],
                rates: vec![1.01, 0.92, 1.19],
            }),
            Rc::new(ClusterFreezePlanAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.91), ("online".to_string(), 0.79)]),
            }),
            Rc::new(PercentFreezePlanAdjuster { percent: 6.1 }),
            Rc::new(MinimumFreezePlanAdjuster { minimum: 16.8 }),
            Rc::new(CapFreezePlanAdjuster { cap: 43768.0 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 59210.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_freeze_plan_engine();
        let item = new_freeze_plan("code-952", "name-540");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
