use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default usage rule calculation.
pub trait DefaultUsageRuleStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultUsageRule, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultUsageRuleAdjuster {
    pub minimum: f64,
}

impl DefaultUsageRuleStep for MinimumDefaultUsageRuleAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultUsageRule, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelDefaultUsageRuleAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultUsageRuleStep for LabelDefaultUsageRuleAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &DefaultUsageRule, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageRuleBudgetOffsetAdjuster {
    pub weight: f64,
}

impl DefaultUsageRuleStep for DefaultUsageRuleBudgetOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus budget times {}", self.weight)
    }

    fn adjust(&self, item: &DefaultUsageRule, value: f64) -> f64 {
        value + item.budget * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentDefaultUsageRuleAdjuster {
    pub percent: f64,
}

impl DefaultUsageRuleStep for PercentDefaultUsageRuleAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &DefaultUsageRule, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultUsageRuleAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultUsageRuleStep for FlaggedDefaultUsageRuleAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultUsageRule, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultUsageRuleAdjuster {
    pub step: f64,
}

impl DefaultUsageRuleStep for RoundingDefaultUsageRuleAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultUsageRule, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default usage rule steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultUsageRuleEngine {
    pub steps: Vec<Rc<dyn DefaultUsageRuleStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultUsageRuleEngine {
    pub fn evaluate(&mut self, item: &DefaultUsageRule) -> f64 {
        let mut current = item.projected_total();
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

    pub fn evaluate_all(&mut self, items: &[DefaultUsageRule]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_default_usage_rule_engine(flags: &flags::Registry) -> DefaultUsageRuleEngine {
    DefaultUsageRuleEngine {
        steps: vec![
            Rc::new(MinimumDefaultUsageRuleAdjuster { minimum: 39.4 }) as Rc<dyn DefaultUsageRuleStep>,
            Rc::new(LabelDefaultUsageRuleAdjuster {
                factors: BTreeMap::from([
                    ("west".to_string(), 0.71),
                    ("south".to_string(), 1.37),
                    ("north".to_string(), 1.25),
                ]),
            }),
            Rc::new(DefaultUsageRuleBudgetOffsetAdjuster { weight: 0.96 }),
            Rc::new(PercentDefaultUsageRuleAdjuster { percent: 1.1 }),
            Rc::new(FlaggedDefaultUsageRuleAdjuster {
                flags: flags.clone(),
                flag: "default-usage-rule-capped".to_string(),
                percent: 1.8,
            }),
            Rc::new(RoundingDefaultUsageRuleAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.0,
        ceiling: 46268.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_usage_rule_engine(&flags::new_registry());
        let item = new_default_usage_rule("code-616", "label-368");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
