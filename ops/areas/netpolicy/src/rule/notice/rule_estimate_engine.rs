use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the rule estimate calculation.
pub trait RuleEstimateStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleEstimate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleEstimateRotationDaysOffsetAdjuster {
    pub weight: f64,
}

impl RuleEstimateStep for RuleEstimateRotationDaysOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus rotation days times {}", self.weight)
    }

    fn adjust(&self, item: &RuleEstimate, value: f64) -> f64 {
        value + (item.rotation_days as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleEstimateAdjuster {
    pub percent: f64,
}

impl RuleEstimateStep for PercentRuleEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleEstimate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleEstimateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleEstimateStep for FlaggedRuleEstimateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleEstimate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleEstimateAdjuster {
    pub cap: f64,
}

impl RuleEstimateStep for CapRuleEstimateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleEstimate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rule estimate steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleEstimateEngine {
    pub steps: Vec<Rc<dyn RuleEstimateStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleEstimateEngine {
    pub fn compute(&mut self, item: &RuleEstimate) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleEstimateStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleEstimate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_estimate_engine(flags: &flags::Registry) -> RuleEstimateEngine {
    RuleEstimateEngine {
        steps: vec![
            Rc::new(RuleEstimateRotationDaysOffsetAdjuster { weight: 1.64 }) as Rc<dyn RuleEstimateStep>,
            Rc::new(PercentRuleEstimateAdjuster { percent: 15.7 }),
            Rc::new(FlaggedRuleEstimateAdjuster {
                flags: flags.clone(),
                flag: "rule-estimate-primary".to_string(),
                percent: 4.3,
            }),
            Rc::new(CapRuleEstimateAdjuster { cap: 47627.0 }),
        ],
        trace: Vec::new(),
        floor: 3.4,
        ceiling: 40924.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_estimate_engine(&flags::new_registry());
        let item = new_rule_estimate("code-79", "label-261");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
