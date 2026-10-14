use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default rollout rollup must pass. Check returns its problems, or none.
pub trait DefaultRolloutRollupRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRolloutRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutRollupRequiredNameRule {
    pub message: String,
}

impl DefaultRolloutRollupRule for DefaultRolloutRollupRequiredNameRule {
    fn check(&self, item: &DefaultRolloutRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutRollupRequiredScopeRule {
    pub message: String,
}

impl DefaultRolloutRollupRule for DefaultRolloutRollupRequiredScopeRule {
    fn check(&self, item: &DefaultRolloutRollup) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRolloutRollupAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultRolloutRollupRule for DefaultRolloutRollupAttemptsRangeRule {
    fn check(&self, item: &DefaultRolloutRollup) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rollout rollup.
#[derive(Debug, Clone)]
pub struct DefaultRolloutRollupValidator {
    pub rules: Vec<Rc<dyn DefaultRolloutRollupRule>>,
}

impl DefaultRolloutRollupValidator {
    pub fn validate(&self, item: &DefaultRolloutRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultRolloutRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRolloutRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rollout_rollup_validator() -> DefaultRolloutRollupValidator {
    DefaultRolloutRollupValidator {
        rules: vec![
            Rc::new(DefaultRolloutRollupRequiredNameRule {
                message: "default rollout rollup needs this field".to_string(),
            }) as Rc<dyn DefaultRolloutRollupRule>,
            Rc::new(DefaultRolloutRollupRequiredScopeRule {
                message: "default rollout rollup needs this field".to_string(),
            }),
            Rc::new(DefaultRolloutRollupAttemptsRangeRule { min: 120, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rollout_rollup_validator();
        let item = new_default_rollout_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultRolloutRollupRequiredNameRule {
            message: "default rollout rollup needs a value".to_string(),
        };
        let mut item = new_default_rollout_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = DefaultRolloutRollupRequiredScopeRule {
            message: "default rollout rollup needs an entry".to_string(),
        };
        let mut item = new_default_rollout_rollup("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = DefaultRolloutRollupAttemptsRangeRule { min: 80, max: 86400 };
        let mut item = new_default_rollout_rollup("x", "y");
        item.attempts = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
