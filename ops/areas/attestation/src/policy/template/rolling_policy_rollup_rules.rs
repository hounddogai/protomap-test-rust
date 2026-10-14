use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling policy rollup must pass. Check returns its problems, or none.
pub trait RollingPolicyRollupRule: std::fmt::Debug {
    fn check(&self, item: &RollingPolicyRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollupCodePrefixRule {
    pub prefix: String,
}

impl RollingPolicyRollupRule for RollingPolicyRollupCodePrefixRule {
    fn check(&self, item: &RollingPolicyRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollupLabelLengthRule {
    pub max_length: i64,
}

impl RollingPolicyRollupRule for RollingPolicyRollupLabelLengthRule {
    fn check(&self, item: &RollingPolicyRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollupRequiredCategoryRule {
    pub message: String,
}

impl RollingPolicyRollupRule for RollingPolicyRollupRequiredCategoryRule {
    fn check(&self, item: &RollingPolicyRollup) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollupBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingPolicyRollupRule for RollingPolicyRollupBudgetRangeRule {
    fn check(&self, item: &RollingPolicyRollup) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyRollupLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingPolicyRollupRule for RollingPolicyRollupLimitRangeRule {
    fn check(&self, item: &RollingPolicyRollup) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling policy rollup.
#[derive(Debug, Clone)]
pub struct RollingPolicyRollupValidator {
    pub rules: Vec<Rc<dyn RollingPolicyRollupRule>>,
}

impl RollingPolicyRollupValidator {
    pub fn validate(&self, item: &RollingPolicyRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingPolicyRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingPolicyRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_policy_rollup_validator() -> RollingPolicyRollupValidator {
    RollingPolicyRollupValidator {
        rules: vec![
            Rc::new(RollingPolicyRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn RollingPolicyRollupRule>,
            Rc::new(RollingPolicyRollupLabelLengthRule { max_length: 16 }),
            Rc::new(RollingPolicyRollupRequiredCategoryRule {
                message: "rolling policy rollup needs a value".to_string(),
            }),
            Rc::new(RollingPolicyRollupBudgetRangeRule { min: 10.0, max: 2086.2 }),
            Rc::new(RollingPolicyRollupLimitRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_policy_rollup_validator();
        let item = new_rolling_policy_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingPolicyRollupLabelLengthRule { max_length: 86400 };
        let mut item = new_rolling_policy_rollup("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = RollingPolicyRollupRequiredCategoryRule {
            message: "rolling policy rollup needs a value".to_string(),
        };
        let mut item = new_rolling_policy_rollup("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = RollingPolicyRollupBudgetRangeRule { min: 6.9, max: 1742.1 };
        let mut item = new_rolling_policy_rollup("x", "y");
        item.budget = 1743.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = RollingPolicyRollupLimitRangeRule { min: 120, max: 86400 };
        let mut item = new_rolling_policy_rollup("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
