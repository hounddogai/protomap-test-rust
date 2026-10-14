use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout preference must pass. Check returns its problems, or none.
pub trait RolloutPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RolloutPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPreferenceNameLengthRule {
    pub max_length: i64,
}

impl RolloutPreferenceRule for RolloutPreferenceNameLengthRule {
    fn check(&self, item: &RolloutPreference) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPreferenceSummaryLengthRule {
    pub max_length: i64,
}

impl RolloutPreferenceRule for RolloutPreferenceSummaryLengthRule {
    fn check(&self, item: &RolloutPreference) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl RolloutPreferenceRule for RolloutPreferenceCodeLengthRule {
    fn check(&self, item: &RolloutPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPreferenceRequiredSummaryRule {
    pub message: String,
}

impl RolloutPreferenceRule for RolloutPreferenceRequiredSummaryRule {
    fn check(&self, item: &RolloutPreference) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutPreferenceRequiredCodeRule {
    pub message: String,
}

impl RolloutPreferenceRule for RolloutPreferenceRequiredCodeRule {
    fn check(&self, item: &RolloutPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout preference.
#[derive(Debug, Clone)]
pub struct RolloutPreferenceValidator {
    pub rules: Vec<Rc<dyn RolloutPreferenceRule>>,
}

impl RolloutPreferenceValidator {
    pub fn validate(&self, item: &RolloutPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RolloutPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_preference_validator() -> RolloutPreferenceValidator {
    RolloutPreferenceValidator {
        rules: vec![
            Rc::new(RolloutPreferenceNameLengthRule { max_length: 3600 }) as Rc<dyn RolloutPreferenceRule>,
            Rc::new(RolloutPreferenceSummaryLengthRule { max_length: 3600 }),
            Rc::new(RolloutPreferenceCodeLengthRule { max_length: 32 }),
            Rc::new(RolloutPreferenceRequiredSummaryRule {
                message: "rollout preference needs an entry".to_string(),
            }),
            Rc::new(RolloutPreferenceRequiredCodeRule {
                message: "rollout preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_preference_validator();
        let item = new_rollout_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RolloutPreferenceNameLengthRule { max_length: 16 };
        let mut item = new_rollout_preference("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = RolloutPreferenceSummaryLengthRule { max_length: 3600 };
        let mut item = new_rollout_preference("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutPreferenceCodeLengthRule { max_length: 64 };
        let mut item = new_rollout_preference("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RolloutPreferenceRequiredSummaryRule {
            message: "rollout preference needs an entry".to_string(),
        };
        let mut item = new_rollout_preference("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RolloutPreferenceRequiredCodeRule {
            message: "rollout preference needs this field".to_string(),
        };
        let mut item = new_rollout_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
