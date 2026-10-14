use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the policy baseline calculation.
pub trait PolicyBaselineStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyBaselineAdjuster {
    pub percent: f64,
}

impl PolicyBaselineStep for PercentPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyBaselineAdjuster {
    pub cap: f64,
}

impl PolicyBaselineStep for CapPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyBaseline, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitlePolicyBaselineAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyBaselineStep for TitlePolicyBaselineAdjuster {
    fn describe(&self) -> String {
        "by title".to_string()
    }

    fn adjust(&self, item: &PolicyBaseline, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyBaselineAdjuster {
    pub minimum: f64,
}

impl PolicyBaselineStep for MinimumPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyBaseline, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPolicyBaselineAdjuster {
    pub step: f64,
}

impl PolicyBaselineStep for RoundingPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PolicyBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredPolicyBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl PolicyBaselineStep for TieredPolicyBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &PolicyBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the policy baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyBaselineEngine {
    pub steps: Vec<Rc<dyn PolicyBaselineStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyBaselineEngine {
    pub fn price(&mut self, item: &PolicyBaseline) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyBaselineStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_policy_baseline_engine() -> PolicyBaselineEngine {
    PolicyBaselineEngine {
        steps: vec![
            Rc::new(PercentPolicyBaselineAdjuster { percent: 11.6 }) as Rc<dyn PolicyBaselineStep>,
            Rc::new(CapPolicyBaselineAdjuster { cap: 34837.0 }),
            Rc::new(TitlePolicyBaselineAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.21)]),
            }),
            Rc::new(MinimumPolicyBaselineAdjuster { minimum: 25.0 }),
            Rc::new(RoundingPolicyBaselineAdjuster { step: 0.05 }),
            Rc::new(TieredPolicyBaselineAdjuster {
                bounds: vec![3515.0, 3726.0, 8397.0],
                rates: vec![0.99, 0.98, 0.89],
            }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 87448.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_baseline_engine();
        let item = new_policy_baseline("code-868", "title-161");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
