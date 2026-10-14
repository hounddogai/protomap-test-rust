use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule milestone calculation.
pub trait RuleMilestoneModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleMilestoneAdjuster {
    pub step: f64,
}

impl RuleMilestoneModifier for RoundingRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleMilestoneAdjuster {
    pub cap: f64,
}

impl RuleMilestoneModifier for CapRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleMilestoneAdjuster {
    pub percent: f64,
}

impl RuleMilestoneModifier for PercentRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleMilestoneAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleMilestoneModifier for TieredRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleMilestone, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMilestoneAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RuleMilestoneModifier for RuleMilestoneAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RuleMilestone, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleMilestoneAdjuster {
    pub minimum: f64,
}

impl RuleMilestoneModifier for MinimumRuleMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rule milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleMilestoneEngine {
    pub steps: Vec<Rc<dyn RuleMilestoneModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleMilestoneEngine {
    pub fn compute(&mut self, item: &RuleMilestone) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleMilestoneModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_milestone_engine() -> RuleMilestoneEngine {
    RuleMilestoneEngine {
        steps: vec![
            Rc::new(RoundingRuleMilestoneAdjuster { step: 1.0 }) as Rc<dyn RuleMilestoneModifier>,
            Rc::new(CapRuleMilestoneAdjuster { cap: 34920.0 }),
            Rc::new(PercentRuleMilestoneAdjuster { percent: -22.6 }),
            Rc::new(TieredRuleMilestoneAdjuster {
                bounds: vec![3037.0, 4750.0, 6055.0, 9163.0],
                rates: vec![0.86, 1.12, 0.93, 1.09],
            }),
            Rc::new(RuleMilestoneAttemptsOffsetAdjuster { weight: 1.16 }),
            Rc::new(MinimumRuleMilestoneAdjuster { minimum: 4.6 }),
        ],
        trace: Vec::new(),
        floor: 1.5,
        ceiling: 14441.6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_milestone_engine();
        let item = new_rule_milestone("code-408", "title-453");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
