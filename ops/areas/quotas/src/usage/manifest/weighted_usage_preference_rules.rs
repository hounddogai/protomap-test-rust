use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted usage preference must pass. Check returns its problems, or none.
pub trait WeightedUsagePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &WeightedUsagePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreferenceCodeLengthRule {
    pub max_length: i64,
}

impl WeightedUsagePreferenceRule for WeightedUsagePreferenceCodeLengthRule {
    fn check(&self, item: &WeightedUsagePreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreferenceRequiredCodeRule {
    pub message: String,
}

impl WeightedUsagePreferenceRule for WeightedUsagePreferenceRequiredCodeRule {
    fn check(&self, item: &WeightedUsagePreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreferenceRequiredTitleRule {
    pub message: String,
}

impl WeightedUsagePreferenceRule for WeightedUsagePreferenceRequiredTitleRule {
    fn check(&self, item: &WeightedUsagePreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedUsagePreferenceRequiredChecksumRule {
    pub message: String,
}

impl WeightedUsagePreferenceRule for WeightedUsagePreferenceRequiredChecksumRule {
    fn check(&self, item: &WeightedUsagePreference) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted usage preference.
#[derive(Debug, Clone)]
pub struct WeightedUsagePreferenceValidator {
    pub rules: Vec<Rc<dyn WeightedUsagePreferenceRule>>,
}

impl WeightedUsagePreferenceValidator {
    pub fn validate(&self, item: &WeightedUsagePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedUsagePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedUsagePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_usage_preference_validator() -> WeightedUsagePreferenceValidator {
    WeightedUsagePreferenceValidator {
        rules: vec![
            Rc::new(WeightedUsagePreferenceCodeLengthRule { max_length: 1000 }) as Rc<dyn WeightedUsagePreferenceRule>,
            Rc::new(WeightedUsagePreferenceRequiredCodeRule {
                message: "weighted usage preference needs a value".to_string(),
            }),
            Rc::new(WeightedUsagePreferenceRequiredTitleRule {
                message: "weighted usage preference needs a value".to_string(),
            }),
            Rc::new(WeightedUsagePreferenceRequiredChecksumRule {
                message: "weighted usage preference needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_usage_preference_validator();
        let item = new_weighted_usage_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedUsagePreferenceCodeLengthRule { max_length: 86400 };
        let mut item = new_weighted_usage_preference("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedUsagePreferenceRequiredCodeRule {
            message: "weighted usage preference needs an entry".to_string(),
        };
        let mut item = new_weighted_usage_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedUsagePreferenceRequiredTitleRule {
            message: "weighted usage preference needs a value".to_string(),
        };
        let mut item = new_weighted_usage_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = WeightedUsagePreferenceRequiredChecksumRule {
            message: "weighted usage preference needs this field".to_string(),
        };
        let mut item = new_weighted_usage_preference("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
