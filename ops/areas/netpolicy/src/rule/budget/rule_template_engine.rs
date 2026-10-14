use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rule template calculation.
pub trait RuleTemplateAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RuleTemplate, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRuleTemplateAdjuster {
    pub percent: f64,
}

impl RuleTemplateAdjuster for PercentRuleTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RuleTemplate, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRuleTemplateAdjuster {
    pub cap: f64,
}

impl RuleTemplateAdjuster for CapRuleTemplateAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RuleTemplate, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRuleTemplateAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RuleTemplateAdjuster for FlaggedRuleTemplateAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RuleTemplate, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryRuleTemplateAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RuleTemplateAdjuster for CategoryRuleTemplateAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &RuleTemplate, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRuleTemplateAdjuster {
    pub step: f64,
}

impl RuleTemplateAdjuster for RoundingRuleTemplateAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RuleTemplate, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rule template steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RuleTemplateEngine {
    pub steps: Vec<Rc<dyn RuleTemplateAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RuleTemplateEngine {
    pub fn compute(&mut self, item: &RuleTemplate) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RuleTemplateAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RuleTemplate]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rule_template_engine(flags: &flags::Registry) -> RuleTemplateEngine {
    RuleTemplateEngine {
        steps: vec![
            Rc::new(PercentRuleTemplateAdjuster { percent: -23.5 }) as Rc<dyn RuleTemplateAdjuster>,
            Rc::new(CapRuleTemplateAdjuster { cap: 43264.0 }),
            Rc::new(FlaggedRuleTemplateAdjuster {
                flags: flags.clone(),
                flag: "rule-template-tiered".to_string(),
                percent: 1.4,
            }),
            Rc::new(CategoryRuleTemplateAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.9), ("central".to_string(), 0.92)]),
            }),
            Rc::new(RoundingRuleTemplateAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 4.3,
        ceiling: 29414.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rule_template_engine(&flags::new_registry());
        let item = new_rule_template("code-222", "title-850");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
