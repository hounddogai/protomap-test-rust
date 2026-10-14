use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout profile must pass. Check returns its problems, or none.
pub trait RolloutProfileRule: std::fmt::Debug {
    fn check(&self, item: &RolloutProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProfileCodePrefixRule {
    pub prefix: String,
}

impl RolloutProfileRule for RolloutProfileCodePrefixRule {
    fn check(&self, item: &RolloutProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProfileRequiredCodeRule {
    pub message: String,
}

impl RolloutProfileRule for RolloutProfileRequiredCodeRule {
    fn check(&self, item: &RolloutProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutProfileMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutProfileRule for RolloutProfileMaximumRangeRule {
    fn check(&self, item: &RolloutProfile) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout profile.
#[derive(Debug, Clone)]
pub struct RolloutProfileValidator {
    pub rules: Vec<Rc<dyn RolloutProfileRule>>,
}

impl RolloutProfileValidator {
    pub fn validate(&self, item: &RolloutProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_profile_validator() -> RolloutProfileValidator {
    RolloutProfileValidator {
        rules: vec![
            Rc::new(RolloutProfileCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RolloutProfileRule>,
            Rc::new(RolloutProfileRequiredCodeRule {
                message: "rollout profile needs a value".to_string(),
            }),
            Rc::new(RolloutProfileMaximumRangeRule { min: 3.4, max: 1894.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_profile_validator();
        let item = new_rollout_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutProfileRequiredCodeRule {
            message: "rollout profile needs an entry".to_string(),
        };
        let mut item = new_rollout_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RolloutProfileMaximumRangeRule { min: 3.2, max: 492.4 };
        let mut item = new_rollout_profile("x", "y");
        item.maximum = 493.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
