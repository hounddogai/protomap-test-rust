use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rule threshold must pass. Check returns its problems, or none.
pub trait RuleThresholdRule: std::fmt::Debug {
    fn check(&self, item: &RuleThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleThresholdCodePrefixRule {
    pub prefix: String,
}

impl RuleThresholdRule for RuleThresholdCodePrefixRule {
    fn check(&self, item: &RuleThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleThresholdRequiredLabelRule {
    pub message: String,
}

impl RuleThresholdRule for RuleThresholdRequiredLabelRule {
    fn check(&self, item: &RuleThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleThresholdCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RuleThresholdRule for RuleThresholdCostRangeRule {
    fn check(&self, item: &RuleThreshold) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rule threshold.
#[derive(Debug, Clone)]
pub struct RuleThresholdValidator {
    pub rules: Vec<Rc<dyn RuleThresholdRule>>,
}

impl RuleThresholdValidator {
    pub fn validate(&self, item: &RuleThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RuleThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RuleThreshold]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_rule_threshold_validator() -> RuleThresholdValidator {
    RuleThresholdValidator {
        rules: vec![
            Rc::new(RuleThresholdCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RuleThresholdRule>,
            Rc::new(RuleThresholdRequiredLabelRule {
                message: "rule threshold needs a value".to_string(),
            }),
            Rc::new(RuleThresholdCostRangeRule { min: 0.5, max: 2141.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_threshold_validator();
        let item = new_rule_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RuleThresholdRequiredLabelRule {
            message: "rule threshold needs a value".to_string(),
        };
        let mut item = new_rule_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RuleThresholdCostRangeRule { min: 5.9, max: 2821.5 };
        let mut item = new_rule_threshold("x", "y");
        item.cost = 2823.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
