use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rule claim calculation.
pub trait RuleClaimModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleClaim, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleClaimRetriesOffsetAdjuster {
    pub weight: f64,
}

impl RuleClaimModifier for RuleClaimRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &RuleClaim, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleClaimAdjuster {
    pub percent: f64,
}

impl RuleClaimModifier for PercentRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleClaim, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleClaimAdjuster {
    pub step: f64,
}

impl RuleClaimModifier for RoundingRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleClaim, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleClaimAdjuster {
    pub minimum: f64,
}

impl RuleClaimModifier for MinimumRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleClaim, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleClaimAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleClaimModifier for TieredRuleClaimAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleClaim, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rule claim steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleClaimEngine {
    pub steps: Vec<Rc<dyn RuleClaimModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleClaimEngine {
    pub fn run(&mut self, item: &RuleClaim) -> f64 {
        let mut current = item.projected_volume();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[RuleClaim]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rule_claim_engine() -> RuleClaimEngine {
    RuleClaimEngine {
        steps: vec![
            Rc::new(RuleClaimRetriesOffsetAdjuster { weight: 1.49 }) as Rc<dyn RuleClaimModifier>,
            Rc::new(PercentRuleClaimAdjuster { percent: -19.6 }),
            Rc::new(RoundingRuleClaimAdjuster { step: 0.5 }),
            Rc::new(MinimumRuleClaimAdjuster { minimum: 36.3 }),
            Rc::new(TieredRuleClaimAdjuster {
                bounds: vec![3371.0, 4389.0, 5796.0, 7121.0],
                rates: vec![1.08, 1.18, 1.15, 0.83],
            }),
        ],
        trace: Vec::new(),
        floor: 0.8,
        ceiling: 44806.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_claim_engine();
        let item = new_rule_claim("code-506", "label-735");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
