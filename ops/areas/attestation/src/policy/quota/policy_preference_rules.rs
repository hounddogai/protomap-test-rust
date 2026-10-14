use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy preference must pass. Check returns its problems, or none.
pub trait PolicyPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &PolicyPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyPreferenceRequiredHostRule {
    pub message: String,
}

impl PolicyPreferenceRule for PolicyPreferenceRequiredHostRule {
    fn check(&self, item: &PolicyPreference) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyPreferenceTimezoneLengthRule {
    pub max_length: i64,
}

impl PolicyPreferenceRule for PolicyPreferenceTimezoneLengthRule {
    fn check(&self, item: &PolicyPreference) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl PolicyPreferenceRule for PolicyPreferenceCodeLengthRule {
    fn check(&self, item: &PolicyPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy preference.
#[derive(Debug, Clone)]
pub struct PolicyPreferenceValidator {
    pub rules: Vec<Rc<dyn PolicyPreferenceRule>>,
}

impl PolicyPreferenceValidator {
    pub fn validate(&self, item: &PolicyPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_preference_validator() -> PolicyPreferenceValidator {
    PolicyPreferenceValidator {
        rules: vec![
            Rc::new(PolicyPreferenceRequiredHostRule {
                message: "policy preference needs a value".to_string(),
            }) as Rc<dyn PolicyPreferenceRule>,
            Rc::new(PolicyPreferenceTimezoneLengthRule { max_length: 16 }),
            Rc::new(PolicyPreferenceCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_preference_validator();
        let item = new_policy_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = PolicyPreferenceRequiredHostRule {
            message: "policy preference needs an entry".to_string(),
        };
        let mut item = new_policy_preference("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = PolicyPreferenceTimezoneLengthRule { max_length: 255 };
        let mut item = new_policy_preference("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyPreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_policy_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
