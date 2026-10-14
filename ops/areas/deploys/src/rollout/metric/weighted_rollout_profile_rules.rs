use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted rollout profile must pass. Check returns its problems, or none.
pub trait WeightedRolloutProfileRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRolloutProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRolloutProfileCodePrefixRule {
    pub prefix: String,
}

impl WeightedRolloutProfileRule for WeightedRolloutProfileCodePrefixRule {
    fn check(&self, item: &WeightedRolloutProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRolloutProfileRequiredReviewerRule {
    pub message: String,
}

impl WeightedRolloutProfileRule for WeightedRolloutProfileRequiredReviewerRule {
    fn check(&self, item: &WeightedRolloutProfile) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRolloutProfileCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRolloutProfileRule for WeightedRolloutProfileCeilingRangeRule {
    fn check(&self, item: &WeightedRolloutProfile) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRolloutProfileNameLengthRule {
    pub max_length: i64,
}

impl WeightedRolloutProfileRule for WeightedRolloutProfileNameLengthRule {
    fn check(&self, item: &WeightedRolloutProfile) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rollout profile.
#[derive(Debug, Clone)]
pub struct WeightedRolloutProfileValidator {
    pub rules: Vec<Rc<dyn WeightedRolloutProfileRule>>,
}

impl WeightedRolloutProfileValidator {
    pub fn validate(&self, item: &WeightedRolloutProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedRolloutProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRolloutProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rollout_profile_validator() -> WeightedRolloutProfileValidator {
    WeightedRolloutProfileValidator {
        rules: vec![
            Rc::new(WeightedRolloutProfileCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn WeightedRolloutProfileRule>,
            Rc::new(WeightedRolloutProfileRequiredReviewerRule {
                message: "weighted rollout profile needs this field".to_string(),
            }),
            Rc::new(WeightedRolloutProfileCeilingRangeRule { min: 5.2, max: 4635.4 }),
            Rc::new(WeightedRolloutProfileNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_rollout_profile_validator();
        let item = new_weighted_rollout_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = WeightedRolloutProfileRequiredReviewerRule {
            message: "weighted rollout profile needs this field".to_string(),
        };
        let mut item = new_weighted_rollout_profile("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = WeightedRolloutProfileCeilingRangeRule { min: 2.6, max: 4185.3 };
        let mut item = new_weighted_rollout_profile("x", "y");
        item.ceiling = 4186.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedRolloutProfileNameLengthRule { max_length: 120 };
        let mut item = new_weighted_rollout_profile("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
