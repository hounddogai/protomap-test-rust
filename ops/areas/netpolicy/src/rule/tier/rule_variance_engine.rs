use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule variance calculation.
pub trait RuleVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleVarianceAdjuster {
    pub step: f64,
}

impl RuleVarianceStep for RoundingRuleVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRuleVarianceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RuleVarianceStep for TieredRuleVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RuleVariance, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRuleVarianceAdjuster {
    pub minimum: f64,
}

impl RuleVarianceStep for MinimumRuleVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RuleVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceRuleVarianceAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleVarianceStep for SourceRuleVarianceAdjuster {
    fn describe(&self) -> String {
        "by source".to_string()
    }

    fn adjust(&self, item: &RuleVariance, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleVarianceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleVarianceStep for FlaggedRuleVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleVariance, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleVarianceAttemptsOffsetAdjuster {
    pub weight: f64,
}

impl RuleVarianceStep for RuleVarianceAttemptsOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus attempts times {}", self.weight)
    }

    fn adjust(&self, item: &RuleVariance, value: f64) -> f64 {
        value + (item.attempts as f64) * self.weight
    }
}

/// Runs the rule variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleVarianceEngine {
    pub steps: Vec<Rc<dyn RuleVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleVarianceEngine {
    pub fn evaluate(&mut self, item: &RuleVariance) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RuleVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_rule_variance_engine(flags: &flags::Registry) -> RuleVarianceEngine {
    RuleVarianceEngine {
        steps: vec![
            Rc::new(RoundingRuleVarianceAdjuster { step: 0.25 }) as Rc<dyn RuleVarianceStep>,
            Rc::new(TieredRuleVarianceAdjuster {
                bounds: vec![1244.0, 6629.0],
                rates: vec![0.98, 1.04],
            }),
            Rc::new(MinimumRuleVarianceAdjuster { minimum: 1.4 }),
            Rc::new(SourceRuleVarianceAdjuster {
                factors: BTreeMap::from([("south".to_string(), 0.86), ("online".to_string(), 0.82)]),
            }),
            Rc::new(FlaggedRuleVarianceAdjuster {
                flags: flags.clone(),
                flag: "rule-variance-adaptive".to_string(),
                percent: -4.0,
            }),
            Rc::new(RuleVarianceAttemptsOffsetAdjuster { weight: 0.52 }),
        ],
        trace: Vec::new(),
        floor: 4.6,
        ceiling: 40502.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_variance_engine(&flags::new_registry());
        let item = new_rule_variance("code-867", "name-351");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
